use std::path::PathBuf;

use chrono::Local;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

/// Creates a zeroed out u8 array of size [`N`].
pub const fn zeroed<const N: usize>() -> [u8; N] {
    let mut arr = [0; N];
    let mut idx = 0;
    while idx < N {
        arr[idx] = 0;
        idx += 1;
    }
    arr
}

/// Extension trait for [`std::path::PathBuf`]s.
pub trait PathBufExt {
    /// Gets the path to this as a [`String`].
    fn to_string(&self) -> String;
}

impl PathBufExt for PathBuf {
    fn to_string(&self) -> String {
        self.to_str().unwrap().to_string()
    }
}

#[macro_export]
macro_rules! wayland_unimplemented {
    () => {
        compiler_error!("Wayland support is currently not implemented");
    };
}

#[macro_export]
macro_rules! catching {
    ($what:literal, $e:expr) => {
        match $e {
            std::result::Result::Ok(o) => o,
            std::result::Result::Err(e) => {
                tracing::error!("{}: {:#?}", $what, e);
                std::process::exit(1)
            }
        }
    };

    (($($what:tt)+), $e:expr) => {
        match $e {
            std::result::Result::Ok(o) => o,
            std::result::Result::Err(e) => {
                tracing::error!("{}: {:#?}", format_args!($($what)+), e);
                std::process::exit(1)
            }
        }
    }
}

#[macro_export]
macro_rules! die {
    ($($body:tt)+) => {
        {
            tracing::error!($($body)+);
            std::process::exit(1);
        }
    };
}

/// A HH:MM:SS time formatter for tracing_subscriber.
pub struct TimeFormatter;

impl FormatTime for TimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let date = Local::now();
        write!(w, "{}", date.format("%H:%M:%S"))
    }
}
