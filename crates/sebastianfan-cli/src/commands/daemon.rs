use sebastianfan_core::Config;
use sebastianfan_daemon::{shutdown_flag, Daemon};
use sebastianfan_sysfs::{find_zone, NitroSense, PlatformProfile};

use crate::cli::LogLevel;
use crate::error::CliError;
use crate::logging;
use crate::paths::Paths;

pub fn run(paths: &Paths, config: &Config, level: LogLevel) -> Result<String, CliError> {
    logging::init(level);
    let sensor = find_zone(&paths.thermal(), &config.sensor)?;
    let control = PlatformProfile::new(&paths.platform_profile());
    let fans = NitroSense::new(&paths.nitro_sense());
    let shutdown = shutdown_flag().map_err(CliError::Signals)?;
    Daemon::new(sensor, control, fans, config).run(&shutdown)?;
    Ok(String::new())
}
