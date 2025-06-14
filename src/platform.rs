use crate::prelude::*;

#[cfg(feature = "x11")]
pub static PLATFORM: platform::X11 = platform::X11;

#[cfg(feature = "x11")]
pub type CurrentPlatform = platform::X11;

#[cfg(not(feature = "x11"))]
wayland_unimplemented!();

/// A platform that rwm can run on. Currently, only X is supported.
/// A platform instance should not hold any data.
pub trait Platform: Plugin + Clone + Copy {
    type State: Resource;

    /// Initialises the platform.
    fn init(world: &mut World);

    /// Manages a window and returns the populated client.
    fn manage(
        window: Window,
        geometry: Geometry,
        commands: &mut Commands,
        state: &mut Self::State,
    ) -> Result<Entity>;
    
    fn unmanage(
        window: Window,
        geometry: Geometry,
        frame: Window,
        monitor: MonitorId,
        commands: &mut Commands,
        state: &mut Self::State,
    ) -> Result<()>;

    /// Updates the position of the given [`client`].
    fn update_client_geometry(
        config: Res<MainConfig>,
        query: Query<(&Geometry, &ClientWindow, &ClientFrame), With<Client>>,
        state: &mut Self::State,
    ) -> Result<()>;
}
