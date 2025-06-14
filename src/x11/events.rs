use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, MapRequestEvent};
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
                        info!("ignored event: {event:#?}");
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

pub fn handle_map_request(mut events: EventReader<X11Event>, mut state: ResMut<X11State>, mut commands: Commands) {
    for event in events.read() {
        if let X11Event::MapRequest(event) = event {
            let geom = &state
                .conn
                .get_geometry(event.window)
                .unwrap()
                .reply()
                .unwrap();

            if let Err(e) = state.manage(
                event.window,
                Geometry::new(
                    geom.x as i32,
                    geom.y as i32,
                    geom.width as u32,
                    geom.height as u32,
                ),
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

// TODO: fix the fucking errors, it still works though (?????????????? repeat like fifty more times)
pub fn handle_unmap_notify(mut events: EventReader<X11Event>, mut state: ResMut<X11State>) {
    for event in events.read() {
        if let X11Event::UnmapNotify(event) = event {
            let Some(client) = state.find_client_owned(event.window) else {
                trace!("tried unmapping nonexistant window: {}", event.window);
                return;
            };
            if let Err(e) = state.unmanage(client) {
                error!("failed unmanaging window {}: {e:?} - this may cause undefined behaviour", event.window);
            }
        }
    }
}

pub fn handle_enter_notify(mut events: EventReader<X11Event>, state: Res<X11State>) {
    for event in events.read() {
        if let X11Event::EnterNotify(event) = event {
            if event.mode != NotifyMode::NORMAL && event.event == state.root_window {
                return;
            }

            let Some(client) = state.find_client(event.event) else {
                trace!("tried entering nonexistant window: {}", event.event);
                return;
            };

            if let Err(e) =
                state
                    .conn
                    .set_input_focus(InputFocus::PARENT, client.window(), CURRENT_TIME)
            {
                warn!(
                    "failed setting input focus to window {}: {e:?} - you may not be able to use your keyboard in this window",
                    event.event
                );
            }

            if let Err(e) = state.conn.configure_window(
                client.frame(),
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

pub fn handle_button_press(mut events: EventReader<X11Event>, mut state: ResMut<X11State>) {
    for event in events.read() {
        if let X11Event::ButtonPress(event) = event {
            if event.detail != 1 || u16::from(event.state) != u16::from(mod_mask()) {
                debug!("skipping attempted drag");
                return;
            }

            if state.find_client(event.event).is_none() {
                trace!("button press in nonexistant window: {}", event.event);
                return;
            };

            if state.dragging.is_none() {
                let (x, y) = (-event.event_x, -event.event_y);
                state.dragging = Some((event.event, (x as i32, y as i32)));
            }
        }
    }
}

pub fn handle_button_release(mut events: EventReader<X11Event>, mut state: ResMut<X11State>) {
    for event in events.read() {
        if let X11Event::ButtonRelease(event) = event {
            if event.detail != 1 {
                state.dragging = None;
            }
        }
    }
}

pub fn handle_motion_notify(mut events: EventReader<X11Event>, mut state: ResMut<X11State>) {
    for event in events.read() {
        if let X11Event::MotionNotify(event) = event {
            let Some((window, (x, y))) = state.dragging else {
                return;
            };

            let (x, y) = (x + event.root_x as i32, y + event.root_y as i32);

            if let Err(e) = state
                .conn
                .configure_window(window, &ConfigureWindowAux::new().x(x).y(y))
            {
                error!("failed moving window {window}: {e:?}");
                return;
            }

            let Some(client) = state.find_client_mut(window) else {
                error!("dragging nonexistant window: {}", window);
                return;
            };

            client.move_to(client.x() + x, client.y() + y);
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

