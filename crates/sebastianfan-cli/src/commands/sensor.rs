use sebastianfan_sysfs::list_zones;

use crate::error::CliError;
use crate::paths::Paths;
use crate::render::{describe, table};

pub fn list(paths: &Paths) -> Result<String, CliError> {
    let rows: Vec<Vec<String>> = list_zones(&paths.thermal())?
        .iter()
        .map(|zone| {
            vec![
                zone.name().to_owned(),
                zone.kind().to_owned(),
                describe(zone.temperature()),
            ]
        })
        .collect();
    Ok(table(&rows))
}
