#[cfg(feature = "x11")]
pub type Window = x11::xlib::Window;

#[cfg(not(feature = "x11"))]
compile_error!("Wayland support is currently nonexistant");
