use bevy::log::{Level, LogPlugin};
use rwm::prelude::*;

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
        .add_event::<KeybindTriggered>()
        .add_systems(
            Startup,
            (print_config, add_fullscreen_remove_handler).chain(),
        )
        .add_systems(
            Update,
            (handle_unmanage, handle_fullscreen, handle_fullscreen_add).chain(),
        )
        .run();
}
