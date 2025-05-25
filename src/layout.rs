use std::fmt::{self, Debug, Formatter};

use crate::prelude::*;

/// A layout manages how clients are layed out in a tag.
pub trait Layout: LayoutClone {
    /// The symbol of this layout displayed in a status bar.
    fn symbol(&self) -> &'static str;

    /// The display name of this layout, used for debugging.
    fn name(&self) -> &'static str;

    /// Manages laying out clients of the given [`tag`] on the given [`monitor`].
    fn manage(&self, tag: Tag, monitor: Monitor);
}

impl Debug for dyn Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ('{}')", self.name(), self.symbol())
    }
}

/// Helper trait for cloning Layout trait objects.
/// Not for public use.
pub trait LayoutClone {
    fn clone_impl(&self) -> Box<dyn Layout>;
}

impl<T> LayoutClone for T
where
    T: 'static + Layout + Clone,
{
    fn clone_impl(&self) -> Box<dyn Layout> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Layout> {
    fn clone(&self) -> Self {
        self.clone_impl()
    }
}
