use derive_getters::Getters;
use derive_setters::Setters;

/// Generic struct for geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Getters, Setters)]
#[setters(prefix = "set_")]
pub struct Geometry {
    /// The x position.
    x: u32,

    /// The y position.
    y: u32,

    /// The width.
    width: u32,

    /// The height.
    height: u32,
}
