use derive_getters::Getters;
use derive_setters::Setters;

use crate::prelude::*;

/// A tag is a workspace that contains any number of clients. By default, only one tag is focused, but any amount of tags can be selected.
#[derive(Debug, Clone, Getters, Setters)]
#[setters(prefix = "set_")]
#[setters(generate = false)]
pub struct Tag {
    /// The index of the tag.
    idx: u8,

    /// The label of the tag.
    label: &'static str,

    /// All clients in this tag.
    clients: Vec<Client>,

    /// The current layout of the tag.
    #[getter(skip)]
    layout: Box<dyn Layout>,

    /// The percentage of the size that the master client is using.
    #[setters(generate)]
    master_factor: f32,

    /// Gaps between windows.
    gaps: Gaps,

    /// The state of this tag.
    state: TagState,
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.label == other.label
    }
}

impl Eq for Tag {}

impl Tag {
    /// A mutable reference to the clients of this tag.
    pub fn clients_mut(&mut self) -> &mut Vec<Client> {
        &mut self.clients
    }

    /// The layout of this tag.
    pub fn layout(&self) -> Box<dyn Layout> {
        self.layout.clone()
    }

    /// Changes the layout of this tag.
    pub fn change_layout<L>(&mut self, layout: L) -> &mut Tag
    where
        L: Layout + 'static,
    {
        self.layout = Box::new(layout);
        self
    }
}

/// Gaps between windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Getters, Setters)]
#[setters(prefix = "set_")]
#[setters(generate_delegates(ty = "Tag", field = "gaps", prefix = "set_gaps_"))]
pub struct Gaps {
    /// Horizontal gap between windows.
    inner_horizontal: u32,

    /// Vertical gaps between windows.
    inner_vertical: u32,

    /// Horizontal outer gaps to the root window.
    outer_horizontal: u32,

    /// Vertical outer gaps to the root window.
    outer_vertical: u32,
}

/// The state of a tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Getters, Setters)]
#[setters(prefix = "set_")]
pub struct TagState {
    /// Whether this tag is currently selected.
    selected: bool,

    /// Whether this tag is currently occupied by one or more clients.
    occupied: bool,

    /// Whether a client has an urgent status in this tag.
    urgent: bool,
}
