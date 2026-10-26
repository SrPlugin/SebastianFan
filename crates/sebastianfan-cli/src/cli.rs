use std::path::PathBuf;

use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{Parser, Subcommand, ValueEnum};
use sebastianfan_core::{ChargeLimit, FanSpeed, FanTarget, Profile};
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
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        default_value = DEFAULT_CONFIG_PATH,
        help = "Path to the configuration file"
    )]
    pub config: PathBuf,

    #[arg(
        long,
        global = true,
        value_name = "DIR",
        default_value = "/",
        help = "Filesystem root used to locate /sys and /etc/systemd"
    )]
    pub root: PathBuf,

    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = LogLevel::Info,
        help = "Verbosity of the daemon log"
    )]
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
    #[command(subcommand, about = "Show the battery and control its charge limit")]
    Battery(BatteryCommand),
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
    #[command(about = "Show the fan control mode and speed of each fan")]
    Get,
    #[command(about = "Set a fan to a percentage (20-100) or back to auto")]
    Set {
        #[arg(value_enum, help = "Fan to control")]
        target: TargetArg,
        #[arg(
            value_name = "SPEED",
            value_parser = |value: &str| value.parse::<FanSpeed>(),
            help = "auto, or a percentage from 20 to 100"
        )]
        speed: FanSpeed,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TargetArg {
    Cpu,
    Gpu,
    All,
}

impl From<TargetArg> for FanTarget {
    fn from(target: TargetArg) -> Self {
        match target {
            TargetArg::Cpu => FanTarget::Cpu,
            TargetArg::Gpu => FanTarget::Gpu,
            TargetArg::All => FanTarget::All,
        }
    }
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
    #[command(about = "Install and start the systemd service")]
    Install,
    #[command(about = "Stop and remove the systemd service")]
    Uninstall,
}

#[derive(Debug, Subcommand)]
pub enum BatteryCommand {
    #[command(about = "Show the battery level, charging state and charge limit")]
    Status,
    #[command(about = "Turn the 80 percent charge limit on or off")]
    Limit {
        #[arg(
            value_name = "STATE",
            value_parser = |value: &str| value.parse::<ChargeLimit>(),
            help = "on or off"
        )]
        state: ChargeLimit,
    },
}
