/// A variant of a cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cursor {
    /// The normal cursor.
    Normal,

    /// The resizing cursor when resizing a window by dragging.
    Resize,

    /// The move cursor when moving a window by dragging.
    Move,

    /// The pencil cursor for when in drawing mode.
    Pencil,
}
