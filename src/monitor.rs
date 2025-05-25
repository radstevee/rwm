use crate::prelude::*;

/// A monitor represents a screen, whether physical or emulated. A monitor can hold [`MAX_TAGS`] amount
/// of tags, which can each hold clients.
#[derive(Clone, Debug, PartialEq, Getters)]
pub struct Monitor {
    /// A tagset containing the selected tags.
    #[getter(copy)]
    selected_tagset: Tagset,

    /// A tagset containing all tags on this monitor.
    #[getter(copy)]
    tagset: Tagset,

    /// All tags on this monitor. These are different to a [`TagSet`] in the way that they are the [`Tag`]
    /// struct instead of its index. The [`Tag`] struct holds information about a tag, such as its clients
    /// and its state.
    tags: [Tag; MAX_TAGS],

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

    /// Gets the tag object for the given [`tag`].
    pub fn tag(&self, tag: usize) -> &Tag {
        &self.tags[tag]
    }
}
