// API

pub use crate::bar::*;
pub use crate::client::*;
pub use crate::color::*;
pub use crate::config::*;
pub use crate::cursor::*;
pub use crate::geometry::*;
pub use crate::layout::*;
pub use crate::monitor::*;
pub use crate::platform::*;
pub use crate::tag::*;
pub use crate::tagset::*;
pub use crate::util::*;
pub use crate::window::*;
pub use crate::cli::*;

// X11 impls
#[cfg(feature = "x11")]
pub use crate::x11::atom::*;
#[cfg(feature = "x11")]
pub use crate::x11::platform::*;
#[cfg(feature = "x11")]
pub use crate::x11::*;

// Macros
pub use crate::catching;
pub use crate::dev_only;
pub use crate::die;
pub use crate::wayland_unimplemented;
pub use crate::wrapper;

// Libraries and utilities
pub use anyhow::{Result, Error, Context, bail};
pub use derive_constructors::*;
pub use derive_getters::*;
pub use derive_setters::*;
pub use bevy::prelude::*;
pub use std::result::Result::*;

pub use crate::platform::PLATFORM;

