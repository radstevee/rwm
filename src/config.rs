use std::{path::PathBuf, sync::OnceLock};

use anyhow::anyhow;
use figment::{
    Error, Figment,
    error::Kind,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

use crate::prelude::*;

/// Main configuration file.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters)]
pub struct MainConfig {
    /// Tag configuration.
    #[serde(default)]
    tags: TagsConfig,

    /// Keyboard configuration.
    #[serde(default)]
    keyboard: KeyboardConfig,

    /// The key bindings.
    #[serde(default)]
    bindings: Vec<KeyBinding>,

    /// Border configuration.
    #[serde(default)]
    border: BorderConfig,
}

impl MainConfig {
    /// Validates all sections of this configuration file.
    pub fn validate(&self) -> Result<()> {
        self.tags.validate()?;
        self.keyboard.validate()?;

        for binding in self.bindings.clone() {
            binding.validate(self.keyboard.mod_key.clone())?;
        }

        self.border.validate()?;

        Ok(())
    }
}

/// Keyboard configuration.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters)]
pub struct KeyboardConfig {
    /// The modifier key.
    mod_key: String,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            mod_key: "super".to_string(),
        }
    }
}

impl KeyboardConfig {
    pub fn validate_modifier(modifier: &str) -> Result<()> {
        if !matches!(
            modifier,
            "super" | "alt" | "ctrl" | "meta" | "windows" | "win"
        ) {
            bail!("invalid modifier key: {modifier}")
        }

        Ok(())
    }

    /// Validates this section.
    pub fn validate(&self) -> Result<()> {
        Self::validate_modifier(&self.mod_key.clone())
    }
}

/// A key binding.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters)]
pub struct KeyBinding {
    /// The key.
    key: String,

    /// The modifier.
    #[serde(rename = "mod")]
    modifiers: Option<Vec<String>>,

    /// A shell command to execute.
    exec: Option<String>,

    /// A tag to switch to.
    switch_tag: Option<u32>,

    /// A tag to move the selected window to.
    move_to_tag: Option<u32>,
}

impl KeyBinding {
    /// Validates this binding.
    pub fn validate(&self, mod_key: String) -> Result<()> {
        if let Some(modifiers) = self.modifiers.clone() {
            for modifier in modifiers {
                KeyboardConfig::validate_modifier(&modifier)?;
            }
        }

        if self.exec.is_none() && self.switch_tag.is_none() && self.move_to_tag.is_none() {
            let mod_key = self
                .modifiers
                .clone()
                .map(|mods| mods.join(" + "))
                .unwrap_or(mod_key);

            warn!(
                "key binding {mod_key} + {} has no actions configured",
                self.key
            )
        }

        Ok(())
    }
}

mod defaults {
    pub fn enabled_tags() -> Vec<u8> {
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
    }
}

/// Configuration element for tags.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters)]
pub struct TagsConfig {
    /// All enabled tags.
    #[serde(default = "defaults::enabled_tags")]
    enabled_tags: Vec<u8>,

    /// Label for tag 1.
    label_1: Option<String>,

    /// Label for tag 2.
    label_2: Option<String>,

    /// Label for tag 3.
    label_3: Option<String>,

    /// Label for tag 4.
    label_4: Option<String>,

    /// Label for tag 5.
    label_5: Option<String>,

    /// Label for tag 6.
    label_6: Option<String>,

    /// Label for tag 7.
    label_7: Option<String>,

    /// Label for tag 8.
    label_8: Option<String>,

    /// Label for tag 9.
    label_9: Option<String>,

    /// Label for tag 10.
    label_10: Option<String>,
}

impl Default for TagsConfig {
    fn default() -> Self {
        Self {
            enabled_tags: defaults::enabled_tags(),
            label_1: None,
            label_2: None,
            label_3: None,
            label_4: None,
            label_5: None,
            label_6: None,
            label_7: None,
            label_8: None,
            label_9: None,
            label_10: None,
        }
    }
}

impl TagsConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled_tags.len() > MAX_TAGS {
            bail!("enabled_tags cannot be larger than {MAX_TAGS}")
        }

        for tag in self.enabled_tags.clone() {
            if tag > MAX_TAGS as u8 || tag < 1 {
                bail!("tag {tag} cannot be below one or above {MAX_TAGS}")
            }

            let label = match tag {
                1 => &self.label_1,
                2 => &self.label_2,
                3 => &self.label_3,
                4 => &self.label_4,
                5 => &self.label_5,
                6 => &self.label_6,
                7 => &self.label_7,
                8 => &self.label_8,
                9 => &self.label_9,
                10 => &self.label_10,
                _ => unreachable!(),
            };

            if label.is_none() {
                bail!("tag {tag} must have a label")
            }
        }

        Ok(())
    }

    /// Gets all labels of all activated tags.
    pub fn labels(&self) -> Vec<String> {
        let mut labels = vec![];

        for tag in self.enabled_tags.clone() {
            let label = match tag {
                1 => &self.label_1,
                2 => &self.label_2,
                3 => &self.label_3,
                4 => &self.label_4,
                5 => &self.label_5,
                6 => &self.label_6,
                7 => &self.label_7,
                8 => &self.label_8,
                9 => &self.label_9,
                10 => &self.label_10,
                _ => unreachable!(),
            };

            if let Some(label) = label {
                labels.push(label.clone());
            }
        }

        labels
    }

    /// Gets the label for the given [`tag`].
    pub fn label(&self, tag: u8) -> Option<String> {
        match tag {
            1 => self.label_1.clone(),
            2 => self.label_2.clone(),
            3 => self.label_3.clone(),
            4 => self.label_4.clone(),
            5 => self.label_5.clone(),
            6 => self.label_6.clone(),
            7 => self.label_7.clone(),
            8 => self.label_8.clone(),
            9 => self.label_9.clone(),
            10 => self.label_10.clone(),
            _ => unreachable!(),
        }
    }
}

/// A colour that can be represented by RGB, hex value or a hex string.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Color {
    Rgb(u8, u8, u8),
    Hex(u32),
    HexString(String),
}

impl Default for Color {
    fn default() -> Self {
        Self::Hex(0xFFFFFF)
    }
}

impl Color {
    /// Gets the hex value of this colour.
    pub fn hex_value(&self) -> Result<u32> {
        match self {
            Self::Rgb(r, g, b) => {
                // Not pretty but eh
                Ok(((*r as u32) << 16) | ((*g as u32) << 8) | (*b as u32))
            }
            Self::Hex(value) => Ok(*value),
            Self::HexString(hex) => {
                let hex = hex.trim_start_matches('#');
                Ok(u32::from_str_radix(hex, 16)?)
            }
        }
    }
}

/// Configuration of window borders.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters, Default)]
pub struct BorderConfig {
    /// Whether the borders are enabled or not.
    #[serde(default)]
    enabled: bool,

    /// The selected border colour.
    #[serde(default)]
    selected_color: Color,

    /// The inactive border colour.
    #[serde(default)]
    inactive_color: Color,

    /// The border width.
    #[serde(default)]
    width: u8,
}

impl BorderConfig {
    /// Validates this configuration section.
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if let Err(e) = self.selected_color.hex_value() {
            bail!("invalid selected_color: {e:?}")
        }

        if let Err(e) = self.inactive_color.hex_value() {
            bail!("invalid inactive_color: {e:?}")
        }

        Ok(())
    }
}

static CONFIG: OnceLock<MainConfig> = OnceLock::new();

/// Loads the configuration from the given configuration file, or `rwm.toml`.
pub fn load_config(file: PathBuf) -> Result<MainConfig> {
    let config = Figment::new()
        .merge(Toml::file(file))
        .merge(Env::prefixed("RWM_"))
        .extract::<MainConfig>()
        .map_err(load_error_friendly)
        .context("failed loading config file")?;

    let _ = CONFIG.set(config.clone());

    Ok(config)
}

fn load_error_friendly(error: Error) -> anyhow::Error {
    let path = if error.path.is_empty() {
        "root"
    } else {
        &error.path.join(".")
    };

    match error.kind {
        Kind::MissingField(field) => anyhow!("missing field at {path}: {field}"),
        Kind::Message(msg) => anyhow!("error at {path}: {msg}"),
        Kind::InvalidType(actual, expected) => {
            anyhow!("invalid type at {path}: {actual}, {expected} was expected")
        }
        Kind::InvalidValue(actual, expected) => {
            anyhow!("invald value at {path}: {actual}, {expected} was expected")
        }
        Kind::InvalidLength(len, expected) => {
            anyhow!("invalid length at {path}: {len}, {expected} was expected")
        }
        Kind::UnknownVariant(actual, expected) => {
            anyhow!("unknown variant at {path}: {actual}, one of {expected:?} was expected")
        }
        Kind::UnknownField(actual, expected) => {
            anyhow!("unknown field at {path}: {actual}, one of {expected:?} was expected")
        }
        Kind::DuplicateField(field) => anyhow!("duplicated field at {path}: {field}"),
        Kind::ISizeOutOfRange(value) => anyhow!("isize out of range at {path}: {value}"),
        Kind::USizeOutOfRange(value) => anyhow!("usize out of range at {path}: {value}"),
        Kind::Unsupported(actual) => anyhow!("unsupported type at {path}: {actual}"),
        Kind::UnsupportedKey(actual, expected) => {
            anyhow!("unsupported key at {path}: {actual}, {expected:?} was expected")
        }
    }
}

/// Gets the loaded configuration file, or panic if it does not exist.
pub fn config() -> &'static MainConfig {
    CONFIG
        .get()
        .unwrap_or_else(|| die!("configuration file not loaded yet"))
}
