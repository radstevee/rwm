use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, MapRequestEvent};
use x11rb::wrapper::ConnectionExt as _;
use x11rb::x11_utils::X11Error;
use x11rb::{
    CURRENT_TIME,
    protocol::{
        Event,
        xproto::{
            ButtonPressEvent, ButtonReleaseEvent, ConfigureWindowAux, EnterNotifyEvent, InputFocus,
            MotionNotifyEvent, NotifyMode, StackMode, UnmapNotifyEvent,
        },
    },
};

use crate::prelude::*;

#[derive(Event, Debug, Clone)]
pub enum X11Event {
    MapRequest(MapRequestEvent),
    UnmapNotify(UnmapNotifyEvent),
    EnterNotify(EnterNotifyEvent),
    MotionNotify(MotionNotifyEvent),
    ButtonPress(ButtonPressEvent),
    ButtonRelease(ButtonReleaseEvent),
    Error(X11Error),
}

pub fn poll_events(mut events: EventWriter<X11Event>, state: Res<X11State>) {
    if let Err(e) = state.conn.flush() {
        error!("failed flushing: {e:?}");
        return;
    }

    loop {
        match state.conn.poll_for_event() {
            Ok(Some(event)) => {
                let x11_event = match event {
                    Event::MapRequest(ev) => Some(X11Event::MapRequest(ev)),
                    Event::UnmapNotify(ev) => Some(X11Event::UnmapNotify(ev)),
                    Event::EnterNotify(ev) => Some(X11Event::EnterNotify(ev)),
                    Event::MotionNotify(ev) => Some(X11Event::MotionNotify(ev)),
                    Event::ButtonPress(ev) => Some(X11Event::ButtonPress(ev)),
                    Event::ButtonRelease(ev) => Some(X11Event::ButtonRelease(ev)),
                    Event::Error(err) => Some(X11Event::Error(err)),
                    _ => {
                        // info!("ignored event: {event:#?}");
                        None
                    }
                };

                if let Some(evt) = x11_event {
                    events.write(evt);
                }
            }
            Ok(None) => break,
            Err(_) => {}
        }
    }
}

pub fn handle_map_request(
    mut events: EventReader<X11Event>,
    mut state: ResMut<X11State>,
    mut monitors: Query<&mut Tags, With<Monitor>>,
    mut commands: Commands,
    main_root: Res<MainRootWindow>,
) {
    for event in events.read() {
        if let X11Event::MapRequest(event) = event {
            dbg!(event);
            let geom = &state
                .conn
                .get_geometry(event.window)
                .unwrap()
                .reply()
                .unwrap();

            for mut tags in &mut monitors {
                let tag = tags.get_mut(0).unwrap(); // TODO: tagging
                if let Err(e) = state.manage(
                    event.window,
                    Geometry::new(
                        geom.x as i32,
                        geom.y as i32,
                        geom.width as u32,
                        geom.height as u32,
                    ),
                    **main_root,
                    tag,
                    &mut commands,
                ) {
                    error!(
                        "failed managing window {}: {e} - this window will not be managed by rwm and may cause undefined behaviour",
                        event.window
                    );
                }
            }
        }
    }
}

// TODO: fix the fucking errors, it still works though (?????????????? repeat like fifty more times)
pub fn handle_unmap_notify(
    mut events: EventReader<X11Event>,
    mut state: ResMut<X11State>,
    mut monitors: Query<&mut Tags, With<Monitor>>,
    query: Query<(Entity, &ClientWindow, &ClientFrame, &Geometry), With<Client>>,
    root_window: Res<MainRootWindow>,
) {
    for event in events.read() {
        if let X11Event::UnmapNotify(event) = event {
            for (client, window, frame, geometry) in query {
                if **window != event.window {
                    continue;
                }
                dbg!(event);

                for mut tags in &mut monitors {
                    let tag = tags.get_mut(0).unwrap();
                    if let Err(e) = state.unmanage(
                        client,
                        **window,
                        *geometry,
                        **frame,
                        **root_window,
                        tag,
                    ) {
                        error!(
                            "failed unmanaging window {}: {e:?} - this may result in undefined behaviour",
                            **window
                        );
                    };
                }
            }
        }
    }
}

pub fn handle_enter_notify(
    mut events: EventReader<X11Event>,
    query: Query<(&ClientWindow, &ClientFrame), With<Client>>,
    state: Res<X11State>,
    main_root: Res<MainRootWindow>,
) {
    for event in events.read() {
        if let X11Event::EnterNotify(event) = event {
            if event.mode != NotifyMode::NORMAL && event.event == **main_root {
                return;
            }

            for (window, frame) in query {
                if event.event != **window {
                    continue;
                }

                if let Err(e) =
                    state
                        .conn
                        .set_input_focus(InputFocus::PARENT, **window, CURRENT_TIME)
                {
                    warn!(
                        "failed setting input focus to window {}: {e:?} - you may not be able to use your keyboard in this window",
                        event.event
                    );
                }

                if let Err(e) = state.conn.configure_window(
                    **frame,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                ) {
                    warn!(
                        "failed setting proper stacking mode for frame window {}: {e:?} - this may cause visual glitches",
                        event.event
                    );
                }

                info!("entered window {}", event.event);
            }
        }
    }
}

pub fn handle_button_press(
    mut events: EventReader<X11Event>,
    mut dragging: ResMut<Dragging>,
    query: Query<(&ClientWindow, &ClientFrame), With<Client>>,
) {
    for event in events.read() {
        if let X11Event::ButtonPress(event) = event {
            if event.detail != 1 || u16::from(event.state) != u16::from(mod_mask()) {
                trace!("skipping attempted drag");
                return;
            }

            debug!("started dragging");

            for (window, frame) in query {
                if event.child != **window
                    && event.child != **frame
                    && event.event != **window
                    && event.event != **frame
                {
                    trace!(
                        "window {} (frame {}) is not event window {}/{}",
                        **window, **frame, event.event, event.child,
                    );
                    continue;
                }

                let (x, y) = (-event.event_x, -event.event_y);
                **dragging = Some((**frame, x, y));
            }
        }
    }
}

pub fn handle_button_release(
    mut events: EventReader<X11Event>,
    mut dragging: ResMut<Dragging>,
    query: Query<(&ClientWindow, &ClientFrame), With<Client>>,
    conn: Res<X11Connection>,
) {
    for event in events.read() {
        if let X11Event::ButtonRelease(event) = event {
            dbg!(event);
            if event.detail != 1 {
                debug!("skipping attempted drag");
                return;
            }

            **dragging = None;
            debug!("stopped dragging");

            for (window, frame) in query {
                if event.child != **window
                    && event.child != **frame
                    && event.event != **window
                    && event.event != **frame
                {
                    trace!(
                        "window {} (frame {}) is not event window {}/{}",
                        **window, **frame, event.event, event.child,
                    );
                    continue;
                }
                conn.ungrab_pointer(CURRENT_TIME).unwrap();

                if let Err(e) = conn.set_input_focus(InputFocus::PARENT, **window, CURRENT_TIME) {
                    warn!(
                        "failed setting input focus to window {}: {e:?} - you may not be able to use your keyboard in this window",
                        event.event
                    );
                }
            }
        }
    }
}

pub fn handle_motion_notify(
    mut events: EventReader<X11Event>,
    mut query: Query<(&mut Geometry, &ClientWindow, &ClientFrame), With<Client>>,
    conn: ResMut<X11Connection>,
    dragging: ResMut<Dragging>,
    config: Res<MainConfig>,
) {
    for event in events.read() {
        if let X11Event::MotionNotify(event) = event {
            if dragging.is_none() {
                continue;
            }
            let Some((_, x, y)) = **dragging else {
                continue;
            };

            for (mut geometry, window, frame) in &mut query {
                if event.child != **window
                    && event.child != **frame
                    && event.event != **window
                    && event.event != **frame
                {
                    trace!(
                        "window {} (frame {}) is not event window {}/{}",
                        **window, **frame, event.event, event.child,
                    );
                    continue;
                }

                let (x, y) = (x + event.root_x, y + event.root_y);
                let (x, y) = (x as i32, y as i32);

                // I have zero fucking clue why this works but setters don't
                geometry.x = x;
                geometry.y = y;

                let border_width = config.border().width() as i16;

                if let Err(e) = conn.configure_window(
                    **frame,
                    &ConfigureWindowAux::new()
                        // what the fuck
                        .x(((geometry.x() as i16) - border_width) as i32)
                        .y(((geometry.y() as i16) - border_width) as i32),
                ) {
                    error!("failed moving window {} whilst dragging: {e:?}", **window);
                };
                conn.sync().unwrap();
            }
        }
    }
}

pub fn handle_error(mut events: EventReader<X11Event>) {
    for event in events.read() {
        if let X11Event::Error(err) = event {
            error!("x11 error: {err:#?}");
        }
    }
}
