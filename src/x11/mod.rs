use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc};

use crate::prelude::*;
use x11rb::{
    COPY_DEPTH_FROM_PARENT,
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt, CreateWindowAux, EventMask, SetMode, WindowClass},
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};

pub mod atom;
pub mod events;
pub mod init;
pub mod keyboard;
pub mod platform;

wrapper!(X11Connection(Arc<RustConnection>));
wrapper!(MainRootWindow(Window));
wrapper!(Dragging(Option<(Window, i16, i16)>));

#[derive(Resource)]
pub struct X11State {
    conn: X11Connection,
    ignored_sequences: BinaryHeap<Reverse<u16>>,
}

impl X11State {
    pub fn window_name(&self, window: Window) -> Result<String> {
        let reply = self
            .conn
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
        &mut self,
        window: Window,
        geometry: Geometry,
        root_window: Window,
        tag: &mut Tag,
        commands: &mut Commands,
    ) -> Result<(Entity, ClientFrame)> {
        info!("Managing window {window} (geom: {geometry:#?}");

        let window_name = self.window_name(window)?.to_string();

        let frame_window = self.conn.generate_id()?;
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
                    .hex_value()
                    .unwrap(),
            );

        let border_width = config.border().width() as i16;
        self.conn.create_window(
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

        self.conn.grab_server()?;
        self.conn.change_save_set(SetMode::INSERT, window)?;
        let cookie = self.conn.reparent_window(
            window,
            frame_window,
            // ??? why the fuck X11
            (border_width as f32 * 1.5) as i16,
            (border_width as f32 * 1.5) as i16,
        )?;
        self.conn.map_window(window)?;
        self.conn.map_window(frame_window)?;
        self.conn.ungrab_server()?;
        self.conn.sync()?;

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

        self.ignored_sequences
            .push(Reverse(cookie.sequence_number() as u16));

        Ok((client, ClientFrame(frame_window)))
    }

    pub fn unmanage(
        &mut self,
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

        self.conn.change_save_set(SetMode::DELETE, window).unwrap();

        self.conn
            .reparent_window(
                window,
                root_window,
                geometry.x() as i16,
                geometry.y() as i16,
            )
            .unwrap();

        if let Some(frame) = frame
            && self
                .conn
                .get_window_attributes(frame)
                .unwrap()
                .reply()
                .is_ok()
        {
            trace!("deleting frame of {window}: {frame}");
            CurrentPlatform::delete_frame(geometry, window, frame, root_window, self);
        }

        self.conn.sync().unwrap();
    }
}

pub(crate) fn flush(conn: Res<X11Connection>) -> Result<()> {
    conn.flush()?;
    Ok(())
}
