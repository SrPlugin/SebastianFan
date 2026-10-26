use sebastianfan_core::{Config, FanControl, ProfileControl};
use sebastianfan_sysfs::{find_zone, list_fans, NitroSense, PlatformProfile};

use crate::paths::Paths;
use crate::render::{describe, table};

pub fn show(paths: &Paths, config: &Config) -> String {
    let mut rows = vec![
        profile_row(paths),
        temperature_row(paths, config),
        fan_control_row(paths),
    ];
    rows.extend(fan_rows(paths));
    table(&rows)
}

fn profile_row(paths: &Paths) -> Vec<String> {
    let current = PlatformProfile::new(&paths.platform_profile()).current();
    vec!["profile".to_owned(), describe(current)]
}

fn fan_control_row(paths: &Paths) -> Vec<String> {
    let setting = NitroSense::new(&paths.nitro_sense()).get();
    vec!["fan control".to_owned(), describe(setting)]
}

fn temperature_row(paths: &Paths, config: &Config) -> Vec<String> {
    let reading = find_zone(&paths.thermal(), &config.sensor)
        .and_then(|zone| zone.temperature())
        .map(|temperature| format!("{temperature} ({})", config.sensor));
    vec!["temperature".to_owned(), describe(reading)]
}

fn fan_rows(paths: &Paths) -> Vec<Vec<String>> {
    match list_fans(&paths.hwmon()) {
        Ok(fans) => fans
            .iter()
            .map(|fan| {
                vec![
                    format!("fan {}/{}", fan.device(), fan.label()),
                    describe(fan.rpm().map(|rpm| format!("{rpm} rpm"))),
                ]
            })
            .collect(),
        Err(error) => vec![vec!["fans".to_owned(), describe::<String>(Err(error))]],
    }
}
