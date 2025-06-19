use crate::prelude::*;
use x11rb::protocol::xproto::InputFocus;
use x11rb::{
    COPY_DEPTH_FROM_PARENT, CURRENT_TIME,
    connection::Connection,
    protocol::xproto::{
        ConfigureWindowAux, ConnectionExt, CreateWindowAux, EventMask, WindowClass,
    },
    wrapper::ConnectionExt as _,
};

#[derive(Clone, Copy)]
pub struct X11;

impl Platform for X11 {
    type State = X11State;

    fn manage(
        window: Window,
        geometry: Geometry,
        root_window: Window,
        tag: &mut Tag,
        commands: &mut Commands,
        state: &mut Self::State,
    ) -> Result<(Entity, ClientFrame)> {
        state.manage(window, geometry, root_window, tag, commands)
    }

    fn unmanage(
        client: Entity,
        window: Window,
        geometry: Geometry,
        frame: Option<Window>,
        root_window: Window,
        tag: &mut Tag,
        commands: &mut Commands,
        state: &mut Self::State,
    ) {
        state.unmanage(client, window, geometry, frame, root_window, commands, tag)
    }

    fn update_bordered_client_geometry(
        config: &MainConfig,
        geometry: Geometry,
        window: Window,
        frame: Window,
        state: &mut Self::State,
    ) {
        let border_width = config.border().width() as i16;

        state
            .conn
            .configure_window(
                window,
                &ConfigureWindowAux::new()
                    // what the fuck
                    .x(((geometry.x() as i16) - border_width) as i32)
                    .y(((geometry.y() as i16) - border_width) as i32)
                    .width(((geometry.width() as i16) + 2 * border_width) as u32)
                    .height(((geometry.height() as i16) + 2 * border_width) as u32),
            )
            .unwrap();

        state
            .conn
            .reparent_window(
                window,
                frame,
                // ??? why the fuck X11
                (border_width as f32 * 1.5) as i16,
                (border_width as f32 * 1.5) as i16,
            )
            .unwrap();

        state.conn.sync().unwrap();
    }

    fn delete_frame(
        geometry: Geometry,
        window: Window,
        frame: Window,
        root_window: Window,
        state: &mut Self::State,
    ) {
        state
            .conn
            .reparent_window(
                window,
                root_window,
                geometry.x() as i16,
                geometry.y() as i16,
            )
            .unwrap();
        state.conn.destroy_window(frame).unwrap();
        state.conn.sync().unwrap();
    }

    fn update_client_geometry(geometry: Geometry, window: Window, state: &mut Self::State) {
        state
            .conn
            .configure_window(
                window,
                &ConfigureWindowAux::new().x(geometry.x()).y(geometry.y()),
            )
            .unwrap();
        state
            .conn
            .configure_window(
                window,
                &ConfigureWindowAux::new()
                    .width(geometry.width())
                    .height(geometry.height()),
            )
            .unwrap();

        state.conn.sync().unwrap();
    }

    fn create_frame(
        config: &MainConfig,
        geometry: Geometry,
        window: Window,
        root_window: Window,
        state: &mut Self::State,
    ) -> Result<ClientFrame> {
        let frame = state.conn.generate_id()?;

        let win_aux = CreateWindowAux::new()
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::SUBSTRUCTURE_NOTIFY
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::ENTER_WINDOW,
            )
            .background_pixel(config.clone().border().selected_color().hex_value()?);

        let border_width = config.border().width() as i16;
        state.conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            frame,
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

        state.conn.reparent_window(
            window,
            frame,
            // ??? why the fuck X11
            (border_width as f32 * 1.5) as i16,
            (border_width as f32 * 1.5) as i16,
        )?;

        state.conn.map_window(frame)?;
        state.conn.map_window(window)?;

        Ok(ClientFrame(frame))
    }

    fn ungrab_mouse(state: &mut Self::State) {
        state.conn.ungrab_pointer(CURRENT_TIME).unwrap();
    }

    fn focus(window: Window, state: &mut Self::State) {
        state
            .conn
            .set_input_focus(InputFocus::PARENT, window, CURRENT_TIME)
            .unwrap();
    }
}

impl Plugin for X11 {
    fn build(&self, app: &mut App) {
        app.add_event::<X11Event>()
            .add_systems(
                Startup,
                (
                    connect,
                    become_wm,
                    load_monitors,
                    init,
                    scan_existing_windows,
                    grab,
                )
                    .chain(),
            )
            .add_systems(First, poll_events)
            .add_systems(
                Update,
                (
                    handle_map_request,
                    handle_unmap_notify,
                    handle_enter_notify,
                    handle_motion_notify,
                    handle_button_press,
                    handle_button_release,
                    handle_key_press,
                    handle_error,
                    flush,
                )
                    .chain(),
            );
    }

    fn name(&self) -> &str {
        "X11"
    }
}
