use derive_getters::Getters;
use derive_setters::Setters;

use crate::prelude::*;

/// The optional status bar.
#[derive(Debug, Clone, Copy, PartialEq, Getters, PartialOrd, Setters)]
#[setters(prefix = "set_")]
pub struct Bar {
    /// The bar window.
    window: Option<Window>,

    /// Whether the bar should be shown and managed by rwm.
    show: bool,

    /// Whether the bar is displayed at the top. Only applies when using the default bar
    /// and not an external one.
    top: bool,
}
