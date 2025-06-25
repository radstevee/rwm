use crate::prelude::*;
use std::sync::Mutex;
use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt, CreateWindowAux, EventMask, SetMode, WindowClass},
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
    COPY_DEPTH_FROM_PARENT,
};

pub mod atom;
pub mod events;
pub mod init;
pub mod keyboard;
pub mod platform;

wrapper!(X11Connection(Arc<RustConnection>));
wrapper!(MainRootWindow(Window));
wrapper!(Dragging(Option<(Window, i16, i16)>));

static IGNORED_SEQUENCES: Mutex<BinaryHeap<Reverse<u16>>> = Mutex::new(BinaryHeap::new());

pub fn window_name(conn: &X11Connection, window: Window) -> Result<String> {
    let reply = conn
        .get_property(
            false,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            0,
            u32::MAX,
        )
        .context("failed getting NET_WM_NAME atom")?
        .reply()
        .context("failed receiving NET_WM_NAME atom")?;

    Ok(String::from_utf8(reply.value)?)
}

pub fn manage(
    conn: &X11Connection,
    window: Window,
    geometry: Geometry,
    root_window: Window,
    tag: &mut Tag,
    commands: &mut Commands,
) -> Result<(Entity, ClientFrame)> {
    info!("Managing window {window} (geom: {geometry:#?}");

    let window_name = window_name(conn, window)?.to_string();

    let frame_window = conn.generate_id()?;
    let config = config();
    let win_aux = CreateWindowAux::new()
        .event_mask(
            EventMask::EXPOSURE
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION
                | EventMask::ENTER_WINDOW,
        )
        .background_pixel(
            config
                .clone()
                .border()
                .selected_color()
                .hex_value()?,
        );

    let border_width = config.border().width() as i16;
    conn.create_window(
        COPY_DEPTH_FROM_PARENT,
        frame_window,
        root_window,
        (geometry.x() as i16) - border_width,
        (geometry.y() as i16) - border_width,
        (geometry.width() as u16) + 2 * border_width as u16,
        (geometry.height() as u16) + 2 * border_width as u16,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &win_aux,
    )?;

    conn.grab_server()?;
    conn.change_save_set(SetMode::INSERT, window)?;
    let cookie = conn.reparent_window(
        window,
        frame_window,
        // ??? why the fuck X11
        (border_width as f32 * 1.5) as i16,
        (border_width as f32 * 1.5) as i16,
    )?;
    conn.map_window(window)?;
    conn.map_window(frame_window)?;
    conn.ungrab_server()?;
    conn.sync()?;

    let client = commands
        .spawn((
            Client,
            ClientName(window_name.clone()),
            geometry,
            OriginalGeometry(geometry),
            ClientWindow(window),
            ClientFrame(frame_window),
            ClientState::default(),
        ))
        .id();

    tag.clients_mut().push(client);

    info!("Managed window {window_name} on tag {}", tag.idx());

    IGNORED_SEQUENCES
        .lock()
        .unwrap()
        .push(Reverse(cookie.sequence_number() as u16));

    Ok((client, ClientFrame(frame_window)))
}

pub fn unmanage(
    conn: &X11Connection,
    client: Entity,
    window: Window,
    geometry: Geometry,
    frame: Option<Window>,
    root_window: Window,
    commands: &mut Commands,
    tag: &mut Tag,
) {
    trace!("unmanaging window {window}");
    tag.clients_mut().retain(|c| &client != c);

    if let Ok(mut entity) = commands.get_entity(client) {
        entity.despawn();
    }

    conn.change_save_set(SetMode::DELETE, window).unwrap();

    conn.reparent_window(
        window,
        root_window,
        geometry.x() as i16,
        geometry.y() as i16,
    )
    .unwrap();

    if let Some(frame) = frame
        && conn.get_window_attributes(frame).unwrap().reply().is_ok()
    {
        trace!("deleting frame of {window}: {frame}");
        RWMP::delete_frame(geometry, window, frame, root_window, conn);
    }

    conn.sync().unwrap();
}

pub(crate) fn flush(conn: Res<X11Connection>) -> Result<()> {
    conn.flush()?;
    Ok(())
}
