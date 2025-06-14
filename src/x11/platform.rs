use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt};

use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct X11;

impl Platform for X11 {
    type State = X11State;

    fn update_client_geometry(
        config: &MainConfig,
        geometry: Geometry,
        window: Window,
        frame: Window,
        state: &mut Self::State,
    ) -> Result<()> {
        let border_width = config.border().width() as i16;
        state.conn.configure_window(
            window,
            &ConfigureWindowAux::new()
                .x(geometry.x())
                .y(geometry.y())
                .width(geometry.width())
                .height(geometry.height()),
        )?;

        state.conn.configure_window(
            window,
            &ConfigureWindowAux::new()
                // what the fuck
                .x(((geometry.x() as i16) - border_width) as i32)
                .y(((geometry.y() as i16) - border_width) as i32)
                .width(((geometry.width() as i16) + 2 * border_width) as u32)
                .height(((geometry.height() as i16) + 2 * border_width) as u32),
        )?;

        state.conn.reparent_window(
            window,
            frame,
            // ??? why the fuck X11
            (border_width as f32 * 1.5) as i16,
            (border_width as f32 * 1.5) as i16,
        )?;
        Ok(())
    }

    fn manage(
        window: Window,
        geometry: Geometry,
        root_window: Window,
        tag: &mut Tag,
        commands: &mut Commands,
        state: &mut Self::State,
    ) -> Result<Entity> {
        state.manage(window, geometry, root_window, tag, commands)
    }

    fn unmanage(
        client: Entity,
        window: Window,
        geometry: Geometry,
        frame: Window,
        root_window: Window,
        tag: &mut Tag,
        state: &mut Self::State,
    ) -> Result<()> {
        state.unmanage(client, window, geometry, frame, root_window, tag)
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
                    handle_error,
                )
                    .chain(),
            );
    }

    fn name(&self) -> &str {
        "X11"
    }
}
