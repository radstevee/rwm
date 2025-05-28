use crate::prelude::*;

/// A monitor represents a screen, whether physical or emulated. A monitor can hold [`MAX_TAGS`] amount
/// of tags, which can each hold clients.
#[derive(Clone, Debug, PartialEq, Getters)]
#[constructor(named(new), fields(idx, tags, dimensions))]
pub struct Monitor {
    /// The index of this monitor.
    #[getter(copy)]
    idx: u8,

    /// The dimensions of this monitor.
    #[getter(copy)]
    dimensions: Geometry,

    /// A tagset containing the selected tags.
    #[getter(copy)]
    selected_tagset: Tagset,

    /// A tagset containing all tags on this monitor.
    #[getter(copy)]
    tagset: Tagset,

    /// All tags on this monitor. These are different to a [`TagSet`] in the way that they are the [`Tag`]
    /// struct instead of its index. The [`Tag`] struct holds information about a tag, such as its clients
    /// and its state.
    tags: Vec<Tag>,

    /// The currently selected client.
    /// On X11, this will be None if currently hovering over the root window.
    selection: Option<&'static Client>,

    /// The last selection.
    last_selection: Option<&'static Client>,

    /// The next monitor in the monitor arrangement.
    next: Option<&'static Monitor>,

    /// The status bar.
    bar: Bar,
}

impl Monitor {
    /// Gets all clients on this monitor.
    pub fn clients(&self) -> Vec<&Client> {
        self.tags.iter().flat_map(Tag::clients).collect()
    }

    /// Gets the tag object at the given [`tag_idx`].
    pub fn tag(&self, tag_idx: usize) -> &Tag {
        &self.tags[tag_idx]
    }

    /// Gets a mutable reference to the tag object at the given [`tag_idx`].
    pub fn tag_mut(&mut self, tag_idx: usize) -> &mut Tag {
        &mut self.tags[tag_idx]
    }
}
