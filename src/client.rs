use crate::prelude::*;

/// A client is a window that rwm manages.
#[derive(Debug, Clone, Copy, PartialEq, Getters, PartialOrd)]
pub struct Client {
    /// The name of this client.
    name: &'static str,

    /// The geometry of this client.
    geometry: Geometry,
    /// The previous geometry of this client.
    prev_geometry: Geometry,

    /// The state of this client.
    state: ClientState,
    /// The previous state of this client.
    prev_state: ClientState,

    /// The size hints of this client. X11 only.
    #[cfg(feature = "x11")]
    size_hints: SizeHints,
}

impl Client {
    /// Creates a new client for the given [`name`] and [`geometry`].
    pub fn new(name: &'static str, geometry: Geometry) -> Client {
        Client {
            name,
            geometry,
            prev_geometry: geometry,
            state: ClientState::default(),
            prev_state: ClientState::default(),
            #[cfg(feature = "x11")]
            size_hints: SizeHints::default(), // TODO
        }
    }

    /// Renames this client to the given [`name`].
    pub fn rename(&mut self, name: &'static str) {
        self.name = name;
    }

    /// The x position of this client.
    pub fn x(&self) -> u32 {
        self.geometry.x()
    }

    /// The previous x position of this client.
    pub fn prev_x(&self) -> u32 {
        self.prev_geometry.x()
    }

    /// The y position of this client.
    pub fn y(&self) -> u32 {
        self.geometry.y()
    }

    /// The previous y position of this client.
    pub fn prev_y(&self) -> u32 {
        self.prev_geometry.y()
    }

    /// The width of this client.
    pub fn width(&self) -> u32 {
        self.geometry.width()
    }

    /// The previous width of this client.
    pub fn prev_width(&self) -> u32 {
        self.prev_geometry.width()
    }

    /// The height of this client.
    pub fn height(&self) -> u32 {
        self.geometry.height()
    }

    /// The previous height of this client.
    pub fn prev_height(&self) -> u32 {
        self.prev_geometry.height()
    }

    /// Resizes this client to the given [`width`] and [`height`].
    pub fn resize(&mut self, width: u32, height: u32) {
        self.prev_geometry.set_width(self.width());
        self.prev_geometry.set_height(self.height());

        self.geometry.set_width(width);
        self.geometry.set_height(height);
    }

    /// Moves this client to the given [`x`] and [`y`] coordinates.
    pub fn move_to(&mut self, x: u32, y: u32) {
        self.prev_geometry.set_x(self.x());
        self.prev_geometry.set_y(self.y());

        self.geometry.set_x(x);
        self.geometry.set_y(y);
    }

    /// Whether this client is in a fixed position.
    pub fn fixed(&self) -> bool {
        self.state.fixed
    }

    /// Sets whether this client is in a fixed position.
    pub fn set_fixed(&mut self, fixed: bool) {
        self.prev_state.fixed = self.fixed();
        self.state.fixed = fixed;
    }

    /// Whether this client was previously in a fixed position.
    pub fn prev_fixed(&self) -> bool {
        self.prev_state.fixed
    }

    /// Whether this client is floating.
    pub fn floating(&self) -> bool {
        self.state.floating
    }

    /// Sets whether this client is floating.
    pub fn set_floating(&mut self, floating: bool) {
        self.prev_state.floating = self.floating();
        self.state.floating = floating;
    }

    /// Whether this client was previously floating.
    pub fn prev_floating(&self) -> bool {
        self.prev_state.floating
    }

    /// Whether this client is in an urgent status.
    pub fn urgent(&self) -> bool {
        self.state.urgent
    }

    /// Sets whether this client is in an urgent status.
    pub fn set_urgent(&mut self, urgent: bool) {
        self.prev_state.urgent = self.urgent();
        self.state.urgent = urgent;
    }

    /// Whether this client was previously in an urgent status.
    pub fn prev_urgent(&self) -> bool {
        self.prev_state.urgent
    }

    /// Whether this client should never focus.
    pub fn never_focus(&self) -> bool {
        self.state.never_focus
    }

    /// Sets whether this client should never focus.
    pub fn set_never_focus(&mut self, never_focus: bool) {
        self.prev_state.never_focus = self.never_focus();
        self.state.never_focus = never_focus;
    }

    /// Whether this client previously could not be focused.
    pub fn prev_never_focus(&self) -> bool {
        self.prev_state.never_focus
    }

    /// Whether this client is fullscreened.
    pub fn fullscreen(&self) -> bool {
        self.state.fullscreen
    }

    /// Sets whether this client is fullscreened.
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.prev_state.fullscreen = self.fullscreen();
        self.state.fullscreen = fullscreen;
    }

    /// Whether this client was previously fullscreened.
    pub fn prev_fullscreen(&self) -> bool {
        self.prev_state.fullscreen
    }

    /// Finds the monitor this client should primarily be located on from the given [`monitor_geoms`]
    /// and returns the index of the geometry in the given [`monitor_geoms`].
    pub fn find_monitor(&self, monitor_geoms: Vec<Geometry>) -> u32 {
        if monitor_geoms.is_empty() {
            return 0;
        }

        monitor_geoms
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let a_overlap = a.overlap(self.geometry);
                let b_overlap = b.overlap(self.geometry);
                a_overlap
                    .partial_cmp(&b_overlap)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx as u32)
            .unwrap_or_default()
    }
}

/// X11 size hints of a client.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Getters, Default)]
pub struct SizeHints {
    /// The base size.
    base: SizeDimensionHint,
    /// The step size.
    step: SizeDimensionHint,

    /// The max size.
    max: SizeDimensionHint,
    /// The min size.
    min: SizeDimensionHint,

    /// The aspect ratio.
    aspect_ratio: SizeConstraintHint,
}

/// A dimension size hint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Getters, Default)]
pub struct SizeDimensionHint {
    /// The width hint.
    width: u32,

    /// The height hint.
    height: u32,
}

/// A size constraint size hint.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Getters, Default)]
pub struct SizeConstraintHint {
    /// The min size.
    min: f32,

    /// The max size.
    max: f32,
}

/// The state of a client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Getters, Default)]
pub struct ClientState {
    /// Whether this client is in a fixed position.
    fixed: bool,

    /// Whether this client is floating.
    floating: bool,

    /// Whether this client is in an urgent status.
    urgent: bool,

    /// Whether this client should never focus.
    never_focus: bool,

    /// Whether this client is fullscreened.
    fullscreen: bool,
}
