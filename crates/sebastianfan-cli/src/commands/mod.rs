mod sensor;

use crate::cli::{Cli, Command, SensorCommand};
use crate::error::CliError;
use crate::paths::Paths;

pub fn run(cli: &Cli) -> Result<String, CliError> {
    let paths = Paths::new(cli.root.clone());
    match &cli.command {
        Command::Sensor(SensorCommand::List) => sensor::list(&paths),
    }
}
