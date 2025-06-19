use crate::prelude::*;

wrapper!(OriginalGeometry(Geometry));

/// Generic struct for geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Getters, Component)]
#[constructor(named(new), fields(x, y, width, height))]
pub struct Geometry {
    /// The x position.
    x: i32,

    /// The y position.
    y: i32,

    /// The width.
    width: u32,

    /// The height.
    height: u32,
}

impl Geometry {
    /// Gets a sum of the x position and width.
    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    /// Gets a sum of the y position and height.
    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    /// Gets the overlap between this geometry object and the [`other`] geometry object.
    pub fn overlap(&self, other: Geometry) -> i32 {
        let x_overlap = self.right().min(other.right()) - self.x.max(other.x);
        let y_overlap = self.bottom().min(other.bottom()) - self.y.max(other.y);
        x_overlap * y_overlap
    }

    /// Whether the given [`x`] and [`y`] points are contained in this geometry object.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}
