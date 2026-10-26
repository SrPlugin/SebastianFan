use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub const DEFAULT_CONFIG_PATH: &str = "/etc/sebastianfan/config.toml";

#[derive(Debug, Parser)]
#[command(
    name = "sebastianfan",
    version,
    about = "Thermal profile control for Linux laptops",
    propagate_version = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[arg(long, global = true, value_name = "PATH", default_value = DEFAULT_CONFIG_PATH)]
    pub config: PathBuf,

    #[arg(long, global = true, value_name = "DIR", default_value = "/")]
    pub root: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(subcommand, about = "Inspect temperature sensors")]
    Sensor(SensorCommand),
}

#[derive(Debug, Subcommand)]
pub enum SensorCommand {
    #[command(about = "List thermal zones with their current temperature")]
    List,
}
