use crate::prelude::*;

/// The maximum amount of tags a monitor can have.
pub const MAX_TAGS: usize = 10;

/// Represents a set of tags that can be toggled between their indices and zeroes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Component)]
pub struct Tagset([u8; MAX_TAGS]);

impl Default for Tagset {
    fn default() -> Self {
        Tagset(zeroed::<MAX_TAGS>())
    }
}

impl Tagset {
    /// All tags in this tagset, regardless of their status.
    pub const fn tags(&self) -> [u8; MAX_TAGS] {
        self.0
    }

    /// Whether the given tag is activated and set to itself.
    pub const fn activated(&self, tag: u8) -> bool {
        self.0[tag as usize] == tag
    }

    /// Activates the given tag.
    pub const fn activate(&mut self, tag: u8) -> &mut Self {
        self.0[tag as usize] = tag;
        self
    }

    /// Deactivates the given tag.
    pub const fn deactivate(&mut self, tag: u8) -> &mut Self {
        self.0[tag as usize] = 0;
        self
    }
}
