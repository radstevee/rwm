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

/// Extension trait to convert things into slices.
pub trait IntoSlice<T> {
    /// Consumes self and returns a slice of the first [`N`] elements.
    fn slice<const N: usize>(self) -> [T; N];
}

impl<T> IntoSlice<T> for Vec<T> {
    fn slice<const N: usize>(self) -> [T; N] {
        self.into_iter()
            .take(N)
            .collect::<Vec<T>>()
            .try_into()
            .unwrap_or_else(|v: Vec<_>| {
                panic!("expected at least {} elements, got only {}", N, v.len())
            })
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
        if let Err(e) = $e {
            tracing::error!("failed {}: {:#?}", $what, e);
            std::process::exit(1)
        }
    };

    (($($what:tt)+), $e:expr) => {
        if let Err(e) = $e {
            tracing::error!("failed {}: {:#?}", format_args!($($what)+), e);
            std::process::exit(1)
        }
    }
}

/// A HH:MM:SS time formatter for tracing_subscriber.
pub struct TimeFormatter;

impl FormatTime for TimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let date = Local::now();
        write!(w, "{}", date.format("%H:%M:%S"))
    }
}
