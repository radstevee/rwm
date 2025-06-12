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
    type State: FromWorld;

    /// Initialises the platform.
    fn init(world: &mut World);
}

