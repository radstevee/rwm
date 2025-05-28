use std::sync::OnceLock;

use crate::prelude::*;
use init::{become_wm, init_state};
use x11rb::{
    COPY_DEPTH_FROM_PARENT, connect,
    connection::Connection,
    protocol::xproto::{
        Atom, AtomEnum, ConnectionExt, CreateWindowAux, EventMask, Gcontext, SetMode, WindowClass,
    },
    rust_connection::RustConnection,
};

pub mod atom;
pub mod init;
pub mod platform;

pub struct X11State {
    conn: RustConnection,
    monitors: Vec<Monitor>,
    root_gc: Gcontext,
    primary_screen: usize,

    wm_protocols: Atom,
    wm_delete_window: Atom,
}

impl X11State {
    pub fn state() -> &'static X11State {
        X11_STATE
            .get()
            .unwrap_or_else(|| die!("X11 state not initialised yet"))
    }

    pub fn create() -> Result<X11State> {
        let (conn, screen_num) = connect(None).context("failed connecting")?;

        let roots = conn.setup().roots.clone();

        let primary_screen = &roots[screen_num];

        become_wm(&conn, primary_screen)?;

        init_state(conn, roots, screen_num)
    }

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

    pub fn manage(&mut self, window: Window, geom: Geometry) -> Result<()> {
        info!("Managing window {window} (geom: {geom:#?}");

        // TODO: fetch window name from NET_WM_NAME
        let window_name = self.window_name(window)?.leak();
        let client = Client::new(window_name, geom); // FIXME: is it fine to leak?
        let monitor_idx =
            client.find_monitor(self.monitors.iter().map(Monitor::dimensions).collect());

        let monitor = self
            .monitors
            .get_mut(monitor_idx as usize)
            .context("monitor was removed whilst starting to manage a window")?;

        let tag = monitor.tag_mut(0); // TODO: tags

        tag.clients_mut().push(client);

        let screen = &self.conn.setup().roots[self.primary_screen];
        let frame_window = self.conn.generate_id()?;
        let win_aux = CreateWindowAux::new()
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::SUBSTRUCTURE_NOTIFY
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::ENTER_WINDOW,
            )
            .background_pixel(screen.white_pixel);
        self.conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            frame_window,
            screen.root,
            geom.x() as i16,
            geom.y() as i16,
            geom.width() as u16,
            geom.height() as u16 + 20,
            1,
            WindowClass::INPUT_OUTPUT,
            0,
            &win_aux,
        ).context("failed creating frame window")?;

        self.conn.grab_server().context("failed grabbing server")?;
        self.conn.change_save_set(SetMode::INSERT, window).context("failed inserting window")?;
        self.conn.reparent_window(window, frame_window, 0, 20).context("failed reparenting window")?;
        self.conn.map_window(window).context("failed mapping window")?;
        self.conn.map_window(frame_window).context("failed mapping parent window")?;
        self.conn.ungrab_server().context("failed ungrabbing server")?;

        info!(
            "Managed window {window_name} on monitor {}, tag {}",
            monitor_idx,
            tag.idx()
        );

        Ok(())
    }
}

static X11_STATE: OnceLock<X11State> = OnceLock::new();
