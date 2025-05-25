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

#[macro_export]
macro_rules! wayland_unimplemented {
    () => {
        compiler_error!("Wayland support is currently not implemented");
    };
}

pub struct TimeFormatter;
impl FormatTime for TimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let date = Local::now();
        write!(w, "{}", date.format("%H:%M:%S"))
    }
}