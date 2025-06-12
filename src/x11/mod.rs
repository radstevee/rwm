use crate::prelude::*;
use init::become_wm;
use keyboard::mod_mask;
use x11rb::{
    connect, connection::Connection, protocol::xproto::{
        AtomEnum, ButtonIndex, ConnectionExt, CreateWindowAux, EventMask, GrabMode, SetMode,
        WindowClass,
    }, rust_connection::RustConnection, wrapper::ConnectionExt as _, COPY_DEPTH_FROM_PARENT
};

pub mod atom;
pub mod events;
pub mod init;
pub mod keyboard;
pub mod platform;

#[derive(Resource)]
pub struct X11State {
    conn: RustConnection,
    monitors: Vec<Monitor>,
    root_window: u32,
    dragging: Option<(Window, (/* x */ u32, /* y */ u32))>,
}

impl FromWorld for X11State {
    fn from_world(world: &mut World) -> Self {
        let (conn, screen_num) = connect(None).unwrap();
        let screens = conn.setup().roots.clone();
        debug!("{screens:#?}");

        let primary_screen = &screens[screen_num];

        become_wm(&conn, primary_screen);

        catching!("failed initialising x11", init::init(world, conn, screens))
    }
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

    pub fn manage(&mut self, window: Window, geom: Geometry) -> Result<Client> {
        info!("Managing window {window} (geom: {geom:#?}");

        let window_name = self.window_name(window)?.leak();
        let monitor_idx = Client::find_monitor(
            geom,
            self.monitors.iter().map(Monitor::dimensions).collect(),
        );

        dbg!(monitor_idx);
        dbg!(&self.monitors);
        let monitor = self
            .monitors
            .get_mut(monitor_idx as usize)
            .context("monitor was removed whilst starting to manage a window")?;

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
        self.conn
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                frame_window,
                self.root_window,
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

        self.conn.grab_server().context("failed grabbing server")?;
        self.conn
            .change_save_set(SetMode::INSERT, window)
            .context("failed inserting window")?;
        self.conn
            .reparent_window(
                window,
                frame_window,
                // ??? why the fuck X11
                (border_width as f32 * 1.5) as i16,
                (border_width as f32 * 1.5) as i16,
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
                self.root_window,
                mod_mask(),
                0, /* any key */
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .context("failed grabbing keys")?;
        self.conn
            .grab_button(
                false,
                self.root_window,
                EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
                self.root_window,
                0u32,
                ButtonIndex::ANY,
                mod_mask(),
            )
            .context("failed grabbing buttons")?;
        self.conn
            .ungrab_server()
            .context("failed ungrabbing server")?;

        self.conn.sync().context("failed synchronising")?;

        let tag = monitor.tag_mut(0); // TODO: tags
        let client = Client::new(window_name, geom, window, frame_window);

        tag.clients_mut().push(client);

        info!(
            "Managed window {window_name} on monitor {}, tag {}",
            monitor_idx,
            tag.idx()
        );

        debug!("{:#?}", self.monitors);

        Ok(client)
    }

    pub fn find_tag_mut(&mut self, client: Client) -> Option<&mut Tag> {
        for monitor in &mut self.monitors {
            for tag in monitor.tags() {
                if !tag.clients().contains(&client) {
                    continue;
                }

                return Some(monitor.tag_mut((tag.idx() - 1) as usize));
            }
        }

        None
    }

    pub fn find_monitor(&self, client: Client) -> Option<&Monitor> {
        for monitor in &self.monitors {
            dbg!(monitor.idx());
            for tag in monitor.tags() {
                dbg!(tag.idx());
                dbg!(tag.clients());
                dbg!(tag);
                if tag.clients().contains(&client) {
                    return Some(monitor);
                }
            }
        }

        None
    }

    pub fn find_monitor_mut(&mut self, client: Client) -> Option<&mut Monitor> {
        for monitor in &mut self.monitors {
            for tag in monitor.tags() {
                if tag.clients().contains(&client) {
                    return Some(monitor);
                }
            }
        }

        None
    }

    pub fn find_client(&self, window: Window) -> Option<&Client> {
        for monitor in &self.monitors {
            for tag in monitor.tags() {
                for client in tag.clients() {
                    if *client.window() == window || *client.frame() == window {
                        return Some(client);
                    }
                }
            }
        }

        None
    }

    pub fn find_client_mut(&mut self, window: Window) -> Option<&mut Client> {
        for monitor in &mut self.monitors {
            for tag in monitor.tags_mut() {
                for client in tag.clients_mut() {
                    if *client.window() == window || *client.frame() == window {
                        return Some(client);
                    }
                }
            }
        }

        None
    }

    pub fn unmanage(&mut self, client: Client) -> Result<()> {
        let window = *client.window();
        let monitor = self.find_monitor(client).unwrap().idx();
        let tag = self
            .find_tag_mut(client)
            .context("client is not on a tag")?;
        tag.clients_mut().retain(|c| &client != c);

        let root = self.conn.setup().roots[monitor as usize].root;

        self.conn
            .change_save_set(SetMode::DELETE, window)
            .context("failed deleting window")?;

        self.conn
            .reparent_window(window, root, client.x() as i16, client.y() as i16)
            .context("failed reparenting window to root")?;

        trace!("destroying frame window: {}", client.frame());
        self.conn
            .destroy_window(*client.frame())
            .context("failed destroying window")?;

        self.conn.sync().context("failed synchronising")?;

        Ok(())
    }
}
