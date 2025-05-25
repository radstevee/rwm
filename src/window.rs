#[cfg(feature = "x11")]
pub type Window = x11rb::protocol::xproto::Window;

#[cfg(not(feature = "x11"))]
crate::wayland_unimplemented!();
