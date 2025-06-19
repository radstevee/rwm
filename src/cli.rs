use crate::prelude::*;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, process::exit};

/// The rwm CLI.
#[derive(Parser, Resource, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The configuration file. Uses rwm.toml in the current working directory by default.
    #[arg(short = 'c', long, value_name = "config")]
    pub config_file: Option<PathBuf>,

    /// The directory where logs should be placed.
    #[arg(short = 'l', long, value_name = "log_dir")]
    pub log_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Prints out the configuration file.
    PrintConfig,
}

impl FromWorld for Cli {
    fn from_world(_world: &mut World) -> Self {
        Self::parse()
    }
}

pub fn print_config(cli: Res<Cli>, config: Res<MainConfig>) {
    if cli.command != Some(CliCommand::PrintConfig) {
        return;
    }

    info!("{config:#?}");
    exit(0)
}
