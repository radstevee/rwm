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
    client: Entity,
}
