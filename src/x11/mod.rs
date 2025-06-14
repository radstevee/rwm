use crate::prelude::*;
use x11rb::{
    COPY_DEPTH_FROM_PARENT, connect,
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ButtonIndex, ConnectionExt, CreateWindowAux, EventMask, GrabMode, SetMode,
        WindowClass,
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};

pub mod atom;
pub mod events;
pub mod init;
pub mod keyboard;
pub mod platform;

wrapper!(X11Connection(RustConnection));
wrapper!(MainRootWindow(Window));

#[derive(Resource)]
pub struct X11State {
    conn: RustConnection,
    dragging: Option<(Window, (/* x */ i32, /* y */ i32))>,
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
        geom: Geometry,
        commands: &mut Commands,
    ) -> Result<Entity> {
        info!("Managing window {window} (geom: {geom:#?}");

        let window_name = self.window_name(window)?.to_string();
        let monitor_idx = find_monitor(
            geom,
            self.monitors.iter().map(Monitor::dimensions).collect(),
        );

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
        let client = commands
            .spawn((
                Client,
                ClientName(window_name.clone()),
                geom,
                ClientWindow(window),
                ClientFrame(frame_window),
                ClientState::default(),
            ))
            .id();

        tag.clients_mut().push(client);

        info!(
            "Managed window {window_name} on monitor {}, tag {}",
            monitor_idx,
            tag.idx()
        );

        debug!("{:#?}", self.monitors);

        Ok(client)
    }

    pub fn find_tag_mut(&mut self, client: Entity) -> Option<&mut Tag> {
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

    pub fn find_monitor(&self, client: Entity) -> Option<&Monitor> {
        for monitor in &self.monitors {
            for tag in monitor.tags() {
                if tag.clients().contains(&client) {
                    return Some(monitor);
                }
            }
        }

        None
    }

    pub fn find_monitor_mut(&mut self, client: Entity) -> Option<&mut Monitor> {
        for monitor in &mut self.monitors {
            for tag in monitor.tags() {
                if tag.clients().contains(&client) {
                    return Some(monitor);
                }
            }
        }

        None
    }

    pub fn unmanage(
        &mut self,
        window: Window,
        geometry: Geometry,
        frame: Window,
        monitor: MonitorId,
        mut commands: Commands,
    ) -> Result<()> {
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
            .reparent_window(window, root, geometry.x() as i16, geometry.y() as i16)
            .context("failed reparenting window to root")?;

        trace!("destroying frame window: {}", frame);
        self.conn
            .destroy_window(frame)
            .context("failed destroying window")?;

        self.conn.sync().context("failed synchronising")?;

        Ok(())
    }
}
