use bevy::log::{Level, LogPlugin};
use rwm::prelude::*;

fn init_platform() {
    dev_only! {
        info!("connecting devtools");
        dioxus_devtools::connect_subsecond();
    };

    info!("starting platform {}", PLATFORM.name());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            #[cfg(debug_assertions)]
            level: Level::TRACE,
            #[cfg(not(debug_assertions))]
            level: Level::INFO,

            ..default()
        }))
        .add_plugins(PLATFORM)
        .init_resource::<Cli>()
        .init_resource::<MainConfig>()
        .add_systems(Startup, init_platform)
        .add_systems(Update, (update_client_geometry, handle_unmanage))
        .run();
}
