use std::path::{Path, PathBuf};

use sebastianfan_core::HardwareError;

use crate::file::{list_entries, numbered_suffix, read_parsed, read_trimmed};

pub const HWMON_DIR: &str = "/sys/class/hwmon";

const DEVICE_PREFIX: &str = "hwmon";
const FAN_PREFIX: &str = "fan";
const FAN_SUFFIX: &str = "_input";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fan {
    device: String,
    label: String,
    input_path: PathBuf,
}

impl Fan {
    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn rpm(&self) -> Result<u32, HardwareError> {
        read_parsed(&self.input_path)
    }
}

fn fan_number(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    name.strip_suffix(FAN_SUFFIX)?
        .strip_prefix(FAN_PREFIX)?
        .parse()
        .ok()
}

fn device_name(device_dir: &Path) -> String {
    read_trimmed(&device_dir.join("name")).unwrap_or_else(|_| {
        device_dir
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
    })
}

fn fans_of(device_dir: &Path) -> Result<Vec<Fan>, HardwareError> {
    let device = device_name(device_dir);
    let mut numbered: Vec<(u32, PathBuf)> = list_entries(device_dir)?
        .into_iter()
        .filter_map(|path| fan_number(&path).map(|number| (number, path)))
        .collect();
    numbered.sort_by_key(|(number, _)| *number);
    Ok(numbered
        .into_iter()
        .map(|(number, input_path)| Fan {
            device: device.clone(),
            label: format!("{FAN_PREFIX}{number}"),
            input_path,
        })
        .collect())
}

pub fn list_fans(dir: &Path) -> Result<Vec<Fan>, HardwareError> {
    let mut devices: Vec<(u32, PathBuf)> = list_entries(dir)?
        .into_iter()
        .filter_map(|path| numbered_suffix(&path, DEVICE_PREFIX).map(|index| (index, path)))
        .collect();
    devices.sort_by_key(|(index, _)| *index);
    let mut fans = Vec::new();
    for (_, device_dir) in devices {
        fans.extend(fans_of(&device_dir)?);
    }
    Ok(fans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::test_support::write_file;

    fn fake_sysfs() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "hwmon3/name", "nvme\n");
        write_file(dir.path(), "hwmon3/temp1_input", "35000\n");
        write_file(dir.path(), "hwmon10/name", "acer\n");
        write_file(dir.path(), "hwmon10/fan2_input", "3307\n");
        write_file(dir.path(), "hwmon10/fan1_input", "3564\n");
        write_file(dir.path(), "hwmon10/fan1_label", "cpu\n");
        write_file(dir.path(), "hwmon2/name", "other\n");
        write_file(dir.path(), "hwmon2/fan1_input", "1000\n");
        dir
    }

    #[test]
    fn lists_fans_ordered_by_device_then_fan_number() {
        let dir = fake_sysfs();
        let fans = list_fans(dir.path()).unwrap();
        let labels: Vec<String> = fans
            .iter()
            .map(|fan| format!("{}/{}", fan.device(), fan.label()))
            .collect();
        assert_eq!(labels, ["other/fan1", "acer/fan1", "acer/fan2"]);
    }

    #[test]
    fn skips_devices_without_fans() {
        let dir = fake_sysfs();
        let fans = list_fans(dir.path()).unwrap();
        assert!(fans.iter().all(|fan| fan.device() != "nvme"));
    }

    #[test]
    fn reads_the_rpm() {
        let dir = fake_sysfs();
        let fans = list_fans(dir.path()).unwrap();
        let rpms: Vec<u32> = fans.iter().map(|fan| fan.rpm().unwrap()).collect();
        assert_eq!(rpms, [1000, 3564, 3307]);
    }

    #[test]
    fn falls_back_to_the_directory_name_without_a_name_file() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "hwmon1/fan1_input", "900\n");
        assert_eq!(list_fans(dir.path()).unwrap()[0].device(), "hwmon1");
    }

    #[test]
    fn reports_malformed_rpm_values() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "hwmon1/fan1_input", "fast\n");
        let fan = list_fans(dir.path()).unwrap().remove(0);
        assert!(matches!(fan.rpm(), Err(HardwareError::Malformed { .. })));
    }

    #[test]
    fn returns_no_fans_for_an_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert!(list_fans(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn fails_when_the_hwmon_directory_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("absent");
        assert!(matches!(list_fans(&missing), Err(HardwareError::Io { .. })));
    }
}
