use crate::prelude::*;

#[cfg(feature = "x11")]
pub mod x11;

#[cfg(feature = "x11")]
pub const PLATFORM: platform::X11 = platform::X11;
#[cfg(not(feature = "x11"))]
wayland_unimplemented!();

/// A platform that rwm can run on. Currently, only X is supported.
pub trait Platform {
    /// The name of the platform.
    fn name(&self) -> &'static str;
    
    /// Initialises the platform.
    fn init(&self) -> Result<()>;
}
