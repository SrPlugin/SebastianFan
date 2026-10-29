use sebastianfan_core::{Config, FanControl};
use sebastianfan_daemon::{shutdown_flag, Daemon};
use sebastianfan_sysfs::{find_zone, NitroSense, PlatformProfile};
use tracing::{info, warn};

use crate::cli::LogLevel;
use crate::error::CliError;
use crate::logging;
use crate::paths::Paths;
use crate::store;

pub fn run(paths: &Paths, config: &Config, level: LogLevel) -> Result<String, CliError> {
    logging::init(level);
    let sensor = find_zone(&paths.thermal(), &config.sensor)?;
    let control = PlatformProfile::new(&paths.platform_profile());
    let fans = NitroSense::new(&paths.nitro_sense());
    restore_fans(paths, &fans);
    let shutdown = shutdown_flag().map_err(CliError::Signals)?;
    Daemon::new(sensor, control, fans, config).run(&shutdown)?;
    Ok(String::new())
}

// The firmware forgets manual fan speeds on reboot, so reapply the last `fan set`.
fn restore_fans(paths: &Paths, fans: &NitroSense) {
    match store::read_fans(&paths.fan_state()) {
        Ok(Some(saved)) => match fans.set(saved) {
            Ok(()) => info!(%saved, "restored saved fan setting"),
            Err(error) => warn!(%error, "cannot restore the saved fan setting"),
        },
        Ok(None) => {}
        Err(error) => warn!(%error, "cannot read the saved fan setting"),
    }
}
