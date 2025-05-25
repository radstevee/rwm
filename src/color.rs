/// A type of colour scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorScheme {
    /// The normal colour scheme.
    Normal,

    /// The colour scheme for when something is selected.
    Selected,

    /// An alternative colour scheme.
    Alt,
}
