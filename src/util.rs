use std::path::PathBuf;

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
            Ok(o) => o,
            Err(e) => {
                error!("{}: {:?}", $what, e);
                std::process::exit(1)
            }
        }
    };

    (($($what:tt)+), $e:expr) => {
        match $e {
            Ok(o) => o,
            Err(e) => {
                error!("{}: {:?}", format_args!($($what)+), e);
                std::process::exit(1)
            }
        }
    }
}

#[macro_export]
macro_rules! die {
    ($($body:tt)+) => {
        {
            error!($($body)+);
            std::process::exit(1);
        }
    };
}

#[macro_export]
macro_rules! dev_only {
    ($($action:tt)+) => {
        #[cfg(debug_assertions)]
        $($action)+
    }
}

#[macro_export]
macro_rules! wrapper {
    ($name:ident($inner:ty)) => {
        #[doc = concat!("A wrapper around [`", stringify!($inner), "`] that can be used as a component or resource.")]
        #[derive(Resource)]
        #[derive(Component)]
        pub struct $name(pub $inner);

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
