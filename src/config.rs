use std::{path::PathBuf, sync::OnceLock};

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

use crate::prelude::*;

/// Main configuration file.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters)]
pub struct MainConfig {
    /// Tag configuration.
    tags: TagsConfig,
}

impl MainConfig {
    /// Validates all sections of this configuration file.
    pub fn validate(&self) -> Result<()> {
        self.tags.validate()
    }
}

/// Configuration element for tags.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters)]
pub struct TagsConfig {
    /// All enabled tags.
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

static CONFIG: OnceLock<MainConfig> = OnceLock::new();

/// Loads the configuration from the given configuration file, or `rwm.toml`.
pub fn load_config(file: PathBuf) -> Result<MainConfig> {
    let config = Figment::new()
        .merge(Toml::file(file))
        .merge(Env::prefixed("RWM_"))
        .extract::<MainConfig>()
        .context("failed loading config file")?;

    let _ = CONFIG.set(config.clone());

    Ok(config)
}

/// Gets the loaded configuration file, or panic if it does not exist.
pub fn config() -> &'static MainConfig {
    CONFIG
        .get()
        .unwrap_or_else(|| die!("configuration file not loaded yet"))
}
