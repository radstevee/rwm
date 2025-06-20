use std::cmp::Reverse;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ConnectionExt as _, KeyPressEvent, MapNotifyEvent, MapRequestEvent, QueryPointerReply,
};
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
    KeyPress(KeyPressEvent),
    MapNotify(MapNotifyEvent),
    Error(X11Error),
}

pub fn poll_events(mut events: EventWriter<X11Event>, mut state: ResMut<X11State>) {
    state.conn.flush().unwrap();

    loop {
        match state.conn.poll_for_event() {
            Ok(Some(event)) => {
                let mut should_ignore = false;
                if let Some(seqno) = event.wire_sequence_number() {
                    // Check sequences_to_ignore and remove entries with old (=smaller) numbers.
                    while let Some(&Reverse(to_ignore)) = state.ignored_sequences.peek() {
                        // Sequence numbers can wrap around, so we cannot simply check for
                        // "to_ignore <= seqno". This is equivalent to "to_ignore - seqno <= 0", which is what we
                        // check instead. Since sequence numbers are unsigned, we need a trick: We decide
                        // that values from [MAX/2, MAX] count as "<= 0" and the rest doesn't.
                        if to_ignore.wrapping_sub(seqno) <= u16::MAX / 2 {
                            // If the two sequence numbers are equal, this event should be ignored.
                            should_ignore = to_ignore == seqno;
                            break;
                        }
                        state.ignored_sequences.pop();
                    }
                }

                if should_ignore {
                    continue;
                }

                let x11_event = match event {
                    Event::MapRequest(ev) => Some(X11Event::MapRequest(ev)),
                    Event::UnmapNotify(ev) => Some(X11Event::UnmapNotify(ev)),
                    Event::EnterNotify(ev) => Some(X11Event::EnterNotify(ev)),
                    Event::MotionNotify(ev) => Some(X11Event::MotionNotify(ev)),
                    Event::ButtonPress(ev) => Some(X11Event::ButtonPress(ev)),
                    Event::ButtonRelease(ev) => Some(X11Event::ButtonRelease(ev)),
                    Event::KeyPress(ev) => Some(X11Event::KeyPress(ev)),
                    Event::MapNotify(ev) => Some(X11Event::MapNotify(ev)),
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
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &ClientWindow,
            Option<&ClientFrame>,
            Has<TransitioningFullscreenStates>,
        ),
        With<Client>,
    >,
) {
    for event in events.read() {
        if let X11Event::UnmapNotify(event) = event {
            for (client, window, frame, transitioning) in query {
                if frame.is_some_and(|f| **f == event.event || **f == event.window) && transitioning
                {
                    continue;
                }

                if **window != event.window {
                    dbg!(**window, event.window);
                    continue;
                }

                debug!("unmanaging: {} {:?}", **window, frame);
                commands.entity(client).insert(Unmanaged);

                if transitioning {
                    commands
                        .entity(client)
                        .remove::<TransitioningFullscreenStates>();
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
                continue;
            }

            for (window, frame) in query {
                if event.event != **window {
                    continue;
                }

                state
                    .conn
                    .set_input_focus(InputFocus::PARENT, **window, CURRENT_TIME)
                    .unwrap();
                state
                    .conn
                    .configure_window(
                        **frame,
                        &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                    )
                    .unwrap();

                info!("entered window {}", event.event);
            }
        }
    }
}

pub fn handle_button_press(
    mut events: EventReader<X11Event>,
    mut dragging: ResMut<Dragging>,
    query: Query<(&ClientWindow, &ClientFrame, Has<Dragging>), With<Client>>,
) {
    for event in events.read() {
        if let X11Event::ButtonPress(event) = event {
            if event.detail != 1 || u16::from(event.state) != u16::from(mod_mask()) {
                trace!("skipping attempted drag");
                continue;
            }

            for (window, frame, is_dragging) in query {
                if event.child != **window
                    && event.child != **frame
                    && event.event != **window
                    && event.event != **frame
                {
                    continue;
                }

                if is_dragging {
                    trace!("cannot drag fullscreened window");
                    continue;
                }

                debug!("started dragging");
                let (x, y) = (-event.event_x, -event.event_y);
                *dragging = Dragging(Some((**frame, x, y)));
            }
        }
    }
}

pub fn handle_button_release(
    mut events: EventReader<X11Event>,
    mut dragging: ResMut<Dragging>,
    query: Query<(&ClientWindow, &ClientFrame, &Geometry), With<Client>>,
    conn: Res<X11Connection>,
    root_window: Res<MainRootWindow>,
) {
    for event in events.read() {
        if let X11Event::ButtonRelease(event) = event {
            if event.detail != 1 {
                debug!("skipping attempted drag");
                continue;
            }

            if dragging.is_none() {
                continue;
            }

            *dragging = Dragging(None);
            debug!("stopped dragging");

            let QueryPointerReply {
                root_x: ptr_x,
                root_y: ptr_y,
                ..
            } = conn.query_pointer(**root_window).unwrap().reply().unwrap();

            for (window, frame, geometry) in query {
                if event.child != **window
                    && event.child != **frame
                    && event.event != **window
                    && event.event != **frame
                {
                    continue;
                }

                if !geometry.contains(ptr_x as i32, ptr_y as i32) {
                    continue;
                }

                conn.ungrab_pointer(CURRENT_TIME).unwrap();

                if let Err(e) = conn.set_input_focus(InputFocus::PARENT, **window, CURRENT_TIME) {
                    warn!(
                        "failed setting input focus to window {}: {e} - you may not be able to use your keyboard in this window",
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
                    continue;
                }

                let (x, y) = (x + event.root_x, y + event.root_y);
                let (x, y) = (x as i32, y as i32);

                *geometry = Geometry::new(x, y, geometry.width(), geometry.height());

                let border_width = config.border().width() as i16;

                if let Err(e) = conn.configure_window(
                    **frame,
                    &ConfigureWindowAux::new()
                        // what the fuck
                        .x(((geometry.x() as i16) - border_width) as i32)
                        .y(((geometry.y() as i16) - border_width) as i32),
                ) {
                    error!("failed moving window {} whilst dragging: {e}", **window);
                };
                conn.sync().unwrap();
            }
        }
    }
}

pub fn handle_key_press(
    mut events: EventReader<X11Event>,
    mut keyboard_events: EventWriter<KeybindTriggered>,
    query: Query<Entity, With<Client>>,
    state: Res<X11State>,
) {
    for event in events.read() {
        if let X11Event::KeyPress(event) = event {
            let Some(action) = find_keybind_action_for(event.detail, event.state, &state) else {
                continue;
            };

            if query.is_empty() {
                keyboard_events.write(KeybindTriggered::new(action.clone(), None));
            }
            for client in query {
                // TODO: focusing
                keyboard_events.write(KeybindTriggered::new(action.clone(), Some(client)));
            }
        }
    }
}

pub fn handle_map_notify(
    mut events: EventReader<X11Event>,
    mut commands: Commands,
    query: Query<(Entity, &ClientFrame, Has<TransitioningFullscreenStates>)>,
) {
    for event in events.read() {
        if let X11Event::MapNotify(event) = event {
            trace!("map notify: {} {}", event.event, event.window);
            for (client, frame, transitioning) in query {
                if **frame == event.event && transitioning {
                    commands.entity(client).remove::<TransitioningFullscreenStates>();
                }
            }
        }
    }
}

pub fn handle_error(mut events: EventReader<X11Event>) {
    for event in events.read() {
        // fuck it for now, we're only logging errors in dev
        if let X11Event::Error(err) = event && cfg!(debug_assertions) {
            error!("x11 error: {err:#?}");
        }
    }
}
