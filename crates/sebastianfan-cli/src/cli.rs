use std::path::PathBuf;

use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{Parser, Subcommand};
use sebastianfan_core::Profile;

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
    #[command(about = "Show the active profile, temperature and fan speeds")]
    Status,
    #[command(subcommand, about = "Inspect temperature sensors")]
    Sensor(SensorCommand),
    #[command(subcommand, about = "Inspect fans")]
    Fan(FanCommand),
    #[command(subcommand, about = "Show or change the platform profile")]
    Profile(ProfileCommand),
    #[command(subcommand, about = "Inspect and validate the temperature curve")]
    Curve(CurveCommand),
}

#[derive(Debug, Subcommand)]
pub enum SensorCommand {
    #[command(about = "List thermal zones with their current temperature")]
    List,
}

#[derive(Debug, Subcommand)]
pub enum FanCommand {
    #[command(about = "List fans with their current speed")]
    List,
}

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    #[command(about = "List the profiles supported by this machine")]
    List,
    #[command(about = "Print the active profile")]
    Get,
    #[command(about = "Activate a profile")]
    Set {
        #[arg(value_parser = profile_parser())]
        profile: Profile,
    },
}

fn profile_parser() -> impl TypedValueParser<Value = Profile> {
    PossibleValuesParser::new(Profile::ALL.map(Profile::as_str))
        .try_map(|name| name.parse::<Profile>())
}

#[derive(Debug, Subcommand)]
pub enum CurveCommand {
    #[command(about = "Print the curve from the active configuration")]
    Show,
    #[command(about = "Check that a configuration file is valid")]
    Validate {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}
