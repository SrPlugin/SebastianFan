use std::path::PathBuf;

use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{Parser, Subcommand, ValueEnum};
use sebastianfan_core::Profile;
use tracing::Level;

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

    #[arg(long, global = true, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Level::ERROR,
            LogLevel::Warn => Level::WARN,
            LogLevel::Info => Level::INFO,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        }
    }
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
    #[command(subcommand, about = "Show or create the configuration file")]
    Config(ConfigCommand),
    #[command(subcommand, about = "Run the background control loop")]
    Daemon(DaemonCommand),
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

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    #[command(about = "Print the path of the configuration file")]
    Path,
    #[command(about = "Print the effective configuration")]
    Show,
    #[command(about = "Write the default configuration file")]
    Init {
        #[arg(long, help = "Overwrite the file if it already exists")]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum DaemonCommand {
    #[command(about = "Run the control loop in the foreground")]
    Run,
}
