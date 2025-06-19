#![feature(let_chains)]

pub mod bar;
pub mod cli;
pub mod client;
pub mod color;
pub mod config;
pub mod cursor;
pub mod geometry;
pub mod keybind;
pub mod layout;
pub mod mgmt;
pub mod monitor;
pub mod tag;
pub mod tagset;
pub mod tiling;
pub mod util;
pub mod window;

pub mod prelude;

pub mod platform;
#[cfg(feature = "x11")]
pub mod x11;
