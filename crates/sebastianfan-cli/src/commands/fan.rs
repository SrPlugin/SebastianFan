use sebastianfan_core::{FanControl, FanSpeed, FanTarget};
use sebastianfan_sysfs::{list_fans, Fan, NitroSense};

use crate::error::CliError;
use crate::paths::Paths;
use crate::render::{describe, table};
use crate::store;

pub fn list(paths: &Paths) -> Result<String, CliError> {
    let rows: Vec<Vec<String>> = list_fans(&paths.hwmon())?
        .iter()
        .map(|fan| {
            vec![
                fan.device().to_owned(),
                fan.label().to_owned(),
                describe(fan.rpm().map(|rpm| format!("{rpm} rpm"))),
            ]
        })
        .collect();
    Ok(table(&rows))
}

fn rpm_cell(fan: Option<&Fan>) -> String {
    fan.map_or_else(
        || "n/a".to_owned(),
        |fan| describe(fan.rpm().map(|rpm| format!("{rpm} rpm"))),
    )
}

pub fn get(paths: &Paths) -> Result<String, CliError> {
    let setting = NitroSense::new(&paths.nitro_sense()).get()?;
    let fans = list_fans(&paths.hwmon()).unwrap_or_default();
    let rows = vec![
        vec![
            "cpu".to_owned(),
            setting.cpu.to_string(),
            rpm_cell(fans.first()),
        ],
        vec![
            "gpu".to_owned(),
            setting.gpu.to_string(),
            rpm_cell(fans.get(1)),
        ],
    ];
    Ok(table(&rows))
}

pub fn set(paths: &Paths, target: FanTarget, speed: FanSpeed) -> Result<String, CliError> {
    let control = NitroSense::new(&paths.nitro_sense());
    let updated = control.get()?.with(target, speed);
    control.set(updated)?;
    store::write_fans(&paths.fan_state(), updated)?;
    Ok(updated.to_string())
}
