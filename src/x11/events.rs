use dioxus_devtools::subsecond::call;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, MapRequestEvent};
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
use crate::x11::keyboard::mod_mask;

fn handle_map_request(state: &mut X11State, event: &MapRequestEvent) -> Result<()> {
    let geom = &state
        .conn
        .get_geometry(event.window)
        .context("failed getting geometry")?
        .reply()
        .context("failed receiving geometry reply")?;

    state
        .manage(
            event.window,
            Geometry::new(
                geom.x as u32,
                geom.y as u32,
                geom.width as u32,
                geom.height as u32,
            ),
        )
        .map(|_| ())
}

// TODO: fix the fucking errors, it still works though (?????????????? repeat like fifty more times)
fn handle_unmap_notify(state: &mut X11State, event: &UnmapNotifyEvent) -> Result<()> {
    let Some(client) = state.find_client(event.window) else {
        trace!("tried unmapping nonexistant window: {}", event.window);
        return Ok(());
    };
    state.unmanage(*client)
}

fn handle_enter(state: &mut X11State, event: &EnterNotifyEvent) -> Result<()> {
    if event.mode != NotifyMode::NORMAL && event.event == state.root_window {
        return Ok(());
    }

    let Some(client) = state.find_client(event.event) else {
        trace!("tried entering nonexistant window: {}", event.event);
        return Ok(());
    };

    state
        .conn
        .set_input_focus(InputFocus::PARENT, *client.window(), CURRENT_TIME)
        .context("failed setting input focus")?;

    state
        .conn
        .configure_window(
            *client.frame(),
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )
        .context("failed configuring frame window to stack above")?;

    info!("entered window {}", event.event);

    Ok(())
}

fn handle_button_press(state: &mut X11State, event: &ButtonPressEvent) -> Result<()> {
    debug!("button press: {:#?}", event.clone());

    if event.detail != 1 || u16::from(event.state) != u16::from(mod_mask()) {
        debug!("skipping attempted drag");
        dbg!(event.detail);
        dbg!(u16::from(event.state));
        dbg!(u16::from(mod_mask()));
        return Ok(());
    }

    // let Some(client) = state.find_client(event.event) else {
    //     trace!("button press in nonexistant window: {}", event.event);
    //     return Ok(());
    // };

    if state.dragging.is_none() {
        let (x, y) = (-event.event_x, -event.event_y);
        state.dragging = Some((event.event, (x as u32, y as u32)));
    }

    Ok(())
}

fn handle_button_release(state: &mut X11State, event: &ButtonReleaseEvent) -> Result<()> {
    if event.detail != 1 {
        state.dragging = None;
    }

    // let Some(client) = state.find_client(event.event) else {
    //     trace!("button release in nonexistant window: {}", event.event);
    //     return Ok(());
    // };
    //
    // if (event.event_x as u32) >= client.width() {
    //     let event = ClientMessageEvent::new(
    //         32,
    //         event.event,
    //         state.wm_protocols,
    //         [state.wm_delete_window, 0, 0, 0, 0],
    //     );
    //
    //     state.conn
    //         .send_event(false, *client.window(), EventMask::NO_EVENT, event)
    //         .context("failed sending delete window event")?;
    // }

    Ok(())
}

fn handle_motion_notify(state: &mut X11State, event: &MotionNotifyEvent) -> Result<()> {
    let Some((window, (x, y))) = state.dragging else {
        return Ok(());
    };
    dbg!(x);
    dbg!(event.root_x);
    dbg!(y);
    dbg!(event.root_y);

    let (x, y) = (x + event.root_x as u32, y + event.root_y as u32);

    state
        .conn
        .configure_window(window, &ConfigureWindowAux::new().x(x as i32).y(y as i32))
        .context("failed configuring window position")?;

    let Some(client) = state.find_client_mut(window) else {
        error!("dragging nonexistant window: {}", window);
        return Ok(());
    };

    client.move_to(x, y);

    Ok(())
}

pub fn handle(state: &mut X11State, event: &Event) -> Result<()> {
    match event {
        Event::MapRequest(ev) => handle_map_request(state, ev)?,
        Event::UnmapNotify(ev) => handle_unmap_notify(state, ev)?,
        Event::Error(err) => error!("x11 error: {err:#?}"),
        Event::EnterNotify(ev) => handle_enter(state, ev)?,
        Event::MotionNotify(ev) => handle_motion_notify(state, ev)?,
        Event::ButtonPress(ev) => handle_button_press(state, ev)?,
        Event::ButtonRelease(ev) => handle_button_release(state, ev)?,
        _ => info!("ignored event: {event:#?}"),
    }

    Ok(())
}

#[derive(Event, Deref)]
pub struct X11Event(pub Event);

pub fn poll(state: Res<X11State>, mut events: EventWriter<X11Event>) {
    if let Err(e) = state.conn.flush() {
        error!("failed flushing: {e:?}");
        return;
    }

    //dbg!(&state.monitors);

    loop {
        match state.conn.poll_for_event() {
            Ok(Some(event)) => {
                events.write(X11Event(event));
            }
            Ok(None) => break,
            Err(_) => {}
        }
    }
}

pub fn handle_events(
    mut events: EventReader<X11Event>,
    mut state: ResMut<X11State>,
) {
    for X11Event(event) in events.read() {
        if let Err(e) = handle(&mut state, event) {
            error!("failed handling event {event:#?}: {e:?}")
        }
    }
}

