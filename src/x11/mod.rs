use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::prelude::*;
use dioxus_devtools::subsecond::call;
use init::{become_wm, init_state};
use keyboard::mod_mask;
use x11rb::{
    connect, connection::Connection, protocol::{
        xproto::{
            Atom, AtomEnum, ConnectionExt, CreateWindowAux, EventMask, Gcontext, GrabMode, MapRequestEvent, SetMode, WindowClass
        }, Event
    }, rust_connection::RustConnection, wrapper::ConnectionExt as _, COPY_DEPTH_FROM_PARENT
};

pub mod atom;
pub mod init;
pub mod keyboard;
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
    pub fn state() -> MutexGuard<'static, X11State> {
        X11_STATE
            .get()
            .unwrap_or_else(|| die!("X11 state not initialised yet"))
            .lock()
            .unwrap_or_else(|e| die!("deadlock: {e:#?}"))
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

        let window_name = self.window_name(window)?.leak();
        let monitor_idx =
            Client::find_monitor(geom, self.monitors.iter().map(Monitor::dimensions).collect());

        let monitor = self
            .monitors
            .get_mut(monitor_idx as usize)
            .context("monitor was removed whilst starting to manage a window")?;

        let screen = &self.conn.setup().roots[self.primary_screen];
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
            .background_pixel(config
                .clone()
                .border()
                .selected_color()
                .hex_value()
                .unwrap()
            );

        let border_width = config.border().width() as i16;
        self.conn
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                frame_window,
                screen.root,
                (geom.x() as i16) - border_width,
                (geom.y() as i16) - border_width,
                (geom.width() as u16) + 2 * border_width as u16,
                (geom.height() as u16) + 2 * border_width as u16,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &win_aux,
            )
            .context("failed creating frame window")?;

        self.conn
            .grab_server()
            .context("failed grabbing server")?;
        self.conn
            .change_save_set(SetMode::INSERT, window)
            .context("failed inserting window")?;
         self.conn
            .reparent_window(
                window,
                frame_window,
                // border_width as i16,
                // border_width as i16
                10, 10
            )
            .context("failed reparenting window")?;
        self.conn
            .map_window(window)
            .context("failed mapping window")?;
        self.conn
            .map_window(frame_window)
            .context("failed mapping frame window")?;
        self.conn
            .grab_key(
                false,
                screen.root,
                mod_mask(),
                0, /* any key */
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .context("failed grabbing keys")?;
        self.conn
            .ungrab_server()
            .context("failed ungrabbing server")?;

        self.conn
            .sync()
            .context("failed synchronising")?;

        let tag = monitor.tag_mut(0); // TODO: tags
        let client = Client::new(window_name, geom, frame_window); // FIXME: is it fine to leak?

        tag.clients_mut().push(client);

        info!(
            "Managed window {window_name} on monitor {}, tag {}",
            monitor_idx,
            tag.idx()
        );

        Ok(())
    }

    pub fn handle_map_request(&mut self, event: &MapRequestEvent) -> Result<()> {
        let geom = &self
            .conn
            .get_geometry(event.window)
            .context("failed getting geometry")?
            .reply()
            .context("failed receiving geometry reply")?;

        self.manage(
            event.window,
            Geometry::new(
                geom.x as u32,
                geom.y as u32,
                geom.width as u32,
                geom.height as u32,
            ),
        )
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<()> {
        info!("event: {:?}", event.clone());
        match event {
            Event::MapRequest(mr) => self.handle_map_request(mr)?,
            Event::KeyPress(key) => info!("key press: {key:#?}"),
            _ => info!("(ignored event)"),
        }

        Ok(())
    }

    pub fn event_loop(&mut self) -> Result<()> {
        self.conn.flush().context("failed flushing")?;

        loop {
            call::<Result<_>>(|| {
                let event = self
                    .conn
                    .wait_for_event()
                    .context("failed waiting for event")?;

                call::<Result<_>>(|| self.handle_event(&event))?;

                while let Some(event) = self
                    .conn
                    .poll_for_event()
                    .context("failed polling for event")?
                {
                    call::<Result<_>>(|| self.handle_event(&event))?;
                }

                Ok(())
            })?
        }
    }
}

static X11_STATE: OnceLock<Mutex<X11State>> = OnceLock::new();
