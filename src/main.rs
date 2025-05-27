use std::path::PathBuf;

use clap::Parser;
use rwm::prelude::*;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt::format};

/// The rwm CLI.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The configuration file. Uses rwm.toml in the current working directory by default.
    #[arg(short = 'c', long, value_name = "config")]
    config_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let fmt = format()
        .with_file(true)
        .with_line_number(true)
        .with_timer(TimeFormatter);

    tracing_subscriber::fmt()
        .event_format(fmt)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(if !cfg!(debug_assertions) {
                    LevelFilter::INFO.into()
                } else {
                    LevelFilter::TRACE.into()
                })
                .with_env_var("RWM_LOG_LEVEL")
                .from_env_lossy(),
        )
        .init();

    let config_file = cli.config_file.unwrap_or(PathBuf::from("rwm.toml"));
    let config_file_path = config_file.to_string();

    info!("loading configuration file {}", config_file_path);
    let config = catching!(
        ("failed loading configuration file: {}", config_file_path),
        load_config(config_file)
    );

    catching!("failed validating configuration file", config.validate());

    debug!("configuration file: {:#?}", config.clone());

    info!("initialising platform {}", PLATFORM.name());
    catching!(
        ("failed initialising platform {}", PLATFORM.name()),
        PLATFORM.init()
    );

    dev_only! {
        dioxus_devtools::connect_subsecond();
    };

    Ok(())
}
