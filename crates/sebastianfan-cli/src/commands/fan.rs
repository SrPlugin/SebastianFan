use sebastianfan_sysfs::list_fans;

use crate::error::CliError;
use crate::paths::Paths;
use crate::render::{describe, table};

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
