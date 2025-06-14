use crate::prelude::*;

#[derive(Clone, Copy, Component)]
pub struct Client;
wrapper!(ClientName(String));
wrapper!(ClientFrame(Window));
wrapper!(ClientWindow(Window));

/// Finds the monitor this client should primarily be located on from the given [`monitor_geoms`]
/// and returns the index of the geometry in the given [`monitor_geoms`].
pub fn find_monitor(client_geometry: Geometry, monitor_geoms: Vec<Geometry>) -> u8 {
    if monitor_geoms.is_empty() {
        return 0;
    }

    monitor_geoms
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let a_overlap = a.overlap(client_geometry);
            let b_overlap = b.overlap(client_geometry);
            a_overlap
                .partial_cmp(&b_overlap)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx as u8)
        .unwrap_or_default()
}

/// X11 size hints of a client.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Getters, Default, Component)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Getters, Default, Component)]
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
