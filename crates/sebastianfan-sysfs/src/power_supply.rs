use std::path::{Path, PathBuf};

use sebastianfan_core::HardwareError;

use crate::file::{list_entries, read_parsed, read_trimmed};

pub const POWER_SUPPLY_DIR: &str = "/sys/class/power_supply";

const BATTERY_TYPE: &str = "Battery";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battery {
    name: String,
    capacity_path: PathBuf,
    status_path: PathBuf,
}

impl Battery {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn capacity(&self) -> Result<u8, HardwareError> {
        read_parsed(&self.capacity_path)
    }

    pub fn status(&self) -> Result<String, HardwareError> {
        read_trimmed(&self.status_path)
    }
}

pub fn find_battery(dir: &Path) -> Result<Battery, HardwareError> {
    let mut entries = list_entries(dir)?;
    entries.sort();
    entries
        .into_iter()
        .find(|path| read_trimmed(&path.join("type")).is_ok_and(|kind| kind == BATTERY_TYPE))
        .map(|path| Battery {
            name: path
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
            capacity_path: path.join("capacity"),
            status_path: path.join("status"),
        })
        .ok_or_else(|| HardwareError::NotFound("battery".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::test_support::write_file;

    fn fake_sysfs() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "ACAD/type", "Mains\n");
        write_file(dir.path(), "BAT1/type", "Battery\n");
        write_file(dir.path(), "BAT1/capacity", "87\n");
        write_file(dir.path(), "BAT1/status", "Charging\n");
        write_file(dir.path(), "ucsi-source-psy-USBC000:001/type", "USB\n");
        dir
    }

    #[test]
    fn finds_the_battery_among_other_power_supplies() {
        let dir = fake_sysfs();
        assert_eq!(find_battery(dir.path()).unwrap().name(), "BAT1");
    }

    #[test]
    fn reads_capacity_and_status() {
        let dir = fake_sysfs();
        let battery = find_battery(dir.path()).unwrap();
        assert_eq!(battery.capacity().unwrap(), 87);
        assert_eq!(battery.status().unwrap(), "Charging");
    }

    #[test]
    fn reports_a_machine_without_a_battery() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "ACAD/type", "Mains\n");
        assert!(matches!(
            find_battery(dir.path()),
            Err(HardwareError::NotFound(_))
        ));
    }

    #[test]
    fn reports_malformed_capacity() {
        let dir = fake_sysfs();
        write_file(dir.path(), "BAT1/capacity", "full\n");
        let battery = find_battery(dir.path()).unwrap();
        assert!(matches!(
            battery.capacity(),
            Err(HardwareError::Malformed { .. })
        ));
    }

    #[test]
    fn fails_when_the_directory_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("absent");
        assert!(matches!(
            find_battery(&missing),
            Err(HardwareError::Io { .. })
        ));
    }
}
