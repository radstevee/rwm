use crate::prelude::*;

#[cfg(feature = "x11")]
pub static PLATFORM: platform::X11 = platform::X11;

#[cfg(feature = "x11")]
pub type CurrentPlatform = platform::X11;

#[cfg(not(feature = "x11"))]
wayland_unimplemented!();

pub type PlatformState = <CurrentPlatform as Platform>::State;

/// A platform that rwm can run on. Currently, only X is supported.
/// A platform instance should not hold any data.
///
/// This is low-level API and should probably not be used directly.
pub trait Platform: Plugin + Clone + Copy {
    type State: Resource;

    /// Manages a window and returns the populated client.
    fn manage(
        window: Window,
        geometry: Geometry,
        root_window: Window,
        tag: &mut Tag,
        commands: &mut Commands,
        state: &mut Self::State,
    ) -> Result<Entity>;

    /// Unmanages a window.
    fn unmanage(
        client: Entity,
        window: Window,
        geometry: Geometry,
        frame: Window,
        root_window: Window,
        tag: &mut Tag,
        state: &mut Self::State,
    ) -> Result<()>;

    /// Updates the position of the given [`client`].
    fn update_client_geometry(
        config: &MainConfig,
        geometry: Geometry,
        window: Window,
        frame: Window,
        state: &mut Self::State,
    ) -> Result<()>;
}
