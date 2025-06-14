use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt};

use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct X11;

impl Platform for X11 {
    type State = X11State;

    fn init(world: &mut World) {
        let monitors = {
            let mut state = world.resource_mut::<Self::State>();
            println!("{:#?}", state.monitors.clone());
            std::mem::take(&mut state.monitors)
        };

        info!("Found {} monitors", monitors.len());
        for monitor in &monitors {
            world.spawn(monitor.clone());
        }

        let mut state = world.resource_mut::<Self::State>();
        state.monitors = monitors;
    }

    fn update_client_geometry(
        config: Res<MainConfig>,
        query: Query<(&Geometry, &ClientWindow, &ClientFrame), With<Client>>,
        state: &mut Self::State,
    ) -> Result<()> {
        let border_width = config.border().width() as i16;
        for (geom, window, frame) in query {
            state.conn.configure_window(
                **window,
                &ConfigureWindowAux::new()
                    .x(geom.x())
                    .y(geom.y())
                    .width(geom.width())
                    .height(geom.height()),
            )?;

            state.conn.configure_window(
                **window,
                &ConfigureWindowAux::new()
                    .x(((geom.x() as i16) - border_width) as i32)
                    .y(((geom.y() as i16) - border_width) as i32)
                    .width(((geom.width() as i16) + 2 * border_width) as u32)
                    .height(((geom.height() as i16) + 2 * border_width) as u32),
            )?;

            state.conn.reparent_window(
                **window,
                **frame,
                // ??? why the fuck X11
                (border_width as f32 * 1.5) as i16,
                (border_width as f32 * 1.5) as i16,
            )?;
        }
        Ok(())
    }

    fn manage(
        window: Window,
        geometry: Geometry,
        commands: &mut Commands,
        state: &mut Self::State,
    ) -> Result<Entity> {
        state.manage(window, geometry, commands)
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
                )
            )
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
                    .after(poll_events),
            );
    }

    fn name(&self) -> &str {
        "X11"
    }
}
