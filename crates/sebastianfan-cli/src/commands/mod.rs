mod fan;
mod profile;
mod sensor;

use crate::cli::{Cli, Command, FanCommand, ProfileCommand, SensorCommand};
use crate::error::CliError;
use crate::paths::Paths;

pub fn run(cli: &Cli) -> Result<String, CliError> {
    let paths = Paths::new(cli.root.clone());
    match &cli.command {
        Command::Sensor(SensorCommand::List) => sensor::list(&paths),
        Command::Fan(FanCommand::List) => fan::list(&paths),
        Command::Profile(ProfileCommand::List) => profile::list(&paths),
        Command::Profile(ProfileCommand::Get) => profile::get(&paths),
        Command::Profile(ProfileCommand::Set { profile }) => profile::set(&paths, *profile),
    }
}
