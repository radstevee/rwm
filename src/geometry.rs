use crate::prelude::*;

/// Generic struct for geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Getters, Setters)]
#[setters(prefix = "set_")]
#[constructor(named(new), fields(x, y, width, height))]
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

impl Geometry {
    /// Gets a sum of the x position and width.
    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    /// Gets a sum of the y position and height.
    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    /// Gets the overlap between this geometry object and the [`other`] geometry object.
    pub fn overlap(&self, other: Geometry) -> u32 {
        let x_overlap = self.right().min(other.right()) - self.x.max(other.x);
        let y_overlap = self.bottom().min(other.bottom()) - self.y.max(other.y);
        x_overlap * y_overlap
    }
}

