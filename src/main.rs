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
        .add_systems(Startup, (init_platform, <CurrentPlatform as Platform>::init).chain())
        .init_resource::<Cli>()
        .init_resource::<MainConfig>()
        .init_resource::<<CurrentPlatform as Platform>::State>()
        .add_plugins(PLATFORM)
        .run();
}
