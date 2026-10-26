use std::path::{Path, PathBuf};

use sebastianfan_core::{HardwareError, SensorSource, Temperature};

use crate::file::{list_entries, numbered_suffix, read_parsed, read_trimmed};

pub const THERMAL_DIR: &str = "/sys/class/thermal";

const ZONE_PREFIX: &str = "thermal_zone";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThermalZone {
    name: String,
    kind: String,
    temperature_path: PathBuf,
}

impl ThermalZone {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn temperature(&self) -> Result<Temperature, HardwareError> {
        read_parsed::<i32>(&self.temperature_path).map(Temperature::from_millicelsius)
    }

    fn load(path: &Path) -> Result<Self, HardwareError> {
        Ok(Self {
            name: path
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
            kind: read_trimmed(&path.join("type"))?,
            temperature_path: path.join("temp"),
        })
    }
}

impl SensorSource for ThermalZone {
    fn read(&self) -> Result<Temperature, HardwareError> {
        self.temperature()
    }
}

pub fn list_zones(dir: &Path) -> Result<Vec<ThermalZone>, HardwareError> {
    let mut indexed: Vec<(u32, PathBuf)> = list_entries(dir)?
        .into_iter()
        .filter_map(|path| numbered_suffix(&path, ZONE_PREFIX).map(|index| (index, path)))
        .collect();
    indexed.sort_by_key(|(index, _)| *index);
    indexed
        .iter()
        .map(|(_, path)| ThermalZone::load(path))
        .collect()
}

pub fn find_zone(dir: &Path, query: &str) -> Result<ThermalZone, HardwareError> {
    list_zones(dir)?
        .into_iter()
        .find(|zone| zone.kind == query || zone.name == query)
        .ok_or_else(|| HardwareError::NotFound(format!("thermal zone {query:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::test_support::write_file;

    fn fake_sysfs() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "thermal_zone0/type", "acpitz\n");
        write_file(dir.path(), "thermal_zone0/temp", "42000\n");
        write_file(dir.path(), "thermal_zone2/type", "TCPU\n");
        write_file(dir.path(), "thermal_zone2/temp", "39050\n");
        write_file(dir.path(), "thermal_zone10/type", "x86_pkg_temp\n");
        write_file(dir.path(), "thermal_zone10/temp", "43000\n");
        write_file(dir.path(), "cooling_device0/type", "Processor\n");
        dir
    }

    #[test]
    fn lists_zones_in_numeric_order() {
        let dir = fake_sysfs();
        let names: Vec<String> = list_zones(dir.path())
            .unwrap()
            .iter()
            .map(|zone| zone.name().to_owned())
            .collect();
        assert_eq!(names, ["thermal_zone0", "thermal_zone2", "thermal_zone10"]);
    }

    #[test]
    fn ignores_entries_that_are_not_zones() {
        let dir = fake_sysfs();
        assert_eq!(list_zones(dir.path()).unwrap().len(), 3);
    }

    #[test]
    fn reads_the_temperature_in_millicelsius() {
        let dir = fake_sysfs();
        let zone = find_zone(dir.path(), "TCPU").unwrap();
        assert_eq!(zone.read().unwrap(), Temperature::from_millicelsius(39_050));
    }

    #[test]
    fn finds_zones_by_directory_name() {
        let dir = fake_sysfs();
        assert_eq!(
            find_zone(dir.path(), "thermal_zone10").unwrap().kind(),
            "x86_pkg_temp"
        );
    }

    #[test]
    fn reports_unknown_zones() {
        let dir = fake_sysfs();
        assert!(matches!(
            find_zone(dir.path(), "GPU"),
            Err(HardwareError::NotFound(_))
        ));
    }

    #[test]
    fn reports_malformed_temperatures() {
        let dir = fake_sysfs();
        write_file(dir.path(), "thermal_zone0/temp", "hot\n");
        let zone = find_zone(dir.path(), "acpitz").unwrap();
        assert!(matches!(
            zone.temperature(),
            Err(HardwareError::Malformed { .. })
        ));
    }

    #[test]
    fn fails_when_the_thermal_directory_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("absent");
        assert!(matches!(
            list_zones(&missing),
            Err(HardwareError::Io { .. })
        ));
    }
}
