mod curve;
mod fan;
mod profile;
mod sensor;
mod status;

use crate::cli::{Cli, Command, CurveCommand, FanCommand, ProfileCommand, SensorCommand};
use crate::error::CliError;
use crate::paths::Paths;
use crate::store;

pub fn run(cli: &Cli) -> Result<String, CliError> {
    let paths = Paths::new(cli.root.clone());
    match &cli.command {
        Command::Status => Ok(status::show(&paths, &store::load(&cli.config)?)),
        Command::Sensor(SensorCommand::List) => sensor::list(&paths),
        Command::Fan(FanCommand::List) => fan::list(&paths),
        Command::Curve(CurveCommand::Show) => Ok(curve::show(&store::load(&cli.config)?)),
        Command::Curve(CurveCommand::Validate { file }) => curve::validate(file),
        Command::Profile(ProfileCommand::List) => profile::list(&paths),
        Command::Profile(ProfileCommand::Get) => profile::get(&paths),
        Command::Profile(ProfileCommand::Set { profile }) => profile::set(&paths, *profile),
    }
}
