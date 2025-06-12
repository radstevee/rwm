use crate::prelude::*;
use clap::Parser;
use std::path::PathBuf;

/// The rwm CLI.
#[derive(Parser, Resource, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The configuration file. Uses rwm.toml in the current working directory by default.
    #[arg(short = 'c', long, value_name = "config")]
    pub config_file: Option<PathBuf>,
}

impl FromWorld for Cli {
    fn from_world(_world: &mut World) -> Self {
        Self::parse()
    }
}
