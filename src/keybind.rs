use std::process::Command;

use serde::Deserialize;

use crate::prelude::*;

/// An action to execute when a keybind is pressed
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum KeybindAction {
    #[serde(rename = "toggle_fullscreen")]
    ToggleFullscreen,

    #[serde(rename = "shell")]
    Shell(String),
}

/// An event that gets triggered when a keybind gets triggered and should execute a [`KeybindAction`].
#[derive(Event, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters, Component)]
#[constructor(named(new), fields(action, client))]
pub struct KeybindTriggered {
    action: KeybindAction,
    client: Option<Entity>,
}

pub fn handle_shell(mut events: EventReader<KeybindTriggered>) {
    for event in events.read() {
        let KeybindAction::Shell(cmd) = event.action() else {
            continue;
        };

        if let Err(e) = Command::new("sh").arg("-c").arg(cmd.clone()).spawn() {
            error!("failed running command {cmd}: {e}");
        }
    }
}
