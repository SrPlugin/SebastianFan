use sebastianfan_core::{BatteryControl, ChargeLimit};
use sebastianfan_sysfs::{find_battery, NitroSense};

use crate::error::CliError;
use crate::paths::Paths;
use crate::render::{describe, table};

pub fn status(paths: &Paths) -> String {
    let level = find_battery(&paths.power_supply()).and_then(|battery| {
        let capacity = battery.capacity()?;
        let state = battery.status()?;
        Ok(format!("{capacity}% ({state})"))
    });
    let limit = NitroSense::new(&paths.nitro_sense()).charge_limit();
    table(&[
        vec!["battery".to_owned(), describe(level)],
        vec!["charge limit".to_owned(), describe(limit)],
    ])
}

pub fn limit(paths: &Paths, state: ChargeLimit) -> Result<String, CliError> {
    NitroSense::new(&paths.nitro_sense()).set_charge_limit(state)?;
    Ok(format!("charge limit {state}"))
}
