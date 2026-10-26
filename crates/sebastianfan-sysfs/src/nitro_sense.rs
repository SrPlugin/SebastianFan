use std::path::{Path, PathBuf};

use sebastianfan_core::{FanControl, FanSetting, FanSpeed, HardwareError, MAX_PERCENT};

use crate::file::{read_trimmed, write_value};

pub const NITRO_SENSE_DIR: &str = "/sys/devices/platform/acer-wmi/nitro_sense";

const FAN_SPEED_FILE: &str = "fan_speed";
const AUTO_RAW: u8 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NitroSense {
    fan_speed_path: PathBuf,
}

impl NitroSense {
    pub fn new(dir: &Path) -> Self {
        Self {
            fan_speed_path: dir.join(FAN_SPEED_FILE),
        }
    }

    pub fn system() -> Self {
        Self::new(Path::new(NITRO_SENSE_DIR))
    }
}

fn parse_speed(text: &str) -> Option<FanSpeed> {
    match text.trim().parse::<u8>().ok()? {
        AUTO_RAW => Some(FanSpeed::Auto),
        percent @ 1..=MAX_PERCENT => Some(FanSpeed::Percent(percent)),
        _ => None,
    }
}

fn raw(speed: FanSpeed) -> u8 {
    match speed {
        FanSpeed::Auto => AUTO_RAW,
        FanSpeed::Percent(percent) => percent,
    }
}

impl FanControl for NitroSense {
    fn get(&self) -> Result<FanSetting, HardwareError> {
        let value = read_trimmed(&self.fan_speed_path)?;
        let parsed = value.split_once(',').and_then(|(cpu, gpu)| {
            Some(FanSetting {
                cpu: parse_speed(cpu)?,
                gpu: parse_speed(gpu)?,
            })
        });
        parsed.ok_or(HardwareError::Malformed {
            path: self.fan_speed_path.clone(),
            value,
        })
    }

    fn set(&self, setting: FanSetting) -> Result<(), HardwareError> {
        write_value(
            &self.fan_speed_path,
            &format!("{},{}", raw(setting.cpu), raw(setting.gpu)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::test_support::write_file;

    fn control_with(content: &str) -> (tempfile::TempDir, NitroSense) {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), FAN_SPEED_FILE, content);
        let control = NitroSense::new(dir.path());
        (dir, control)
    }

    #[test]
    fn reads_automatic_mode() {
        let (_dir, control) = control_with("0,0\n");
        assert_eq!(control.get().unwrap(), FanSetting::AUTO);
    }

    #[test]
    fn reads_manual_percentages() {
        let (_dir, control) = control_with("60,45\n");
        let expected = FanSetting {
            cpu: FanSpeed::Percent(60),
            gpu: FanSpeed::Percent(45),
        };
        assert_eq!(control.get().unwrap(), expected);
    }

    #[test]
    fn reads_values_below_the_manual_minimum_set_by_other_tools() {
        let (_dir, control) = control_with("10,0\n");
        assert_eq!(control.get().unwrap().cpu, FanSpeed::Percent(10));
    }

    #[test]
    fn rejects_malformed_content() {
        for content in ["", "60", "60;60", "a,b", "101,0", "-1,0", "60,60,60"] {
            let (_dir, control) = control_with(content);
            assert!(
                matches!(control.get(), Err(HardwareError::Malformed { .. })),
                "{content}"
            );
        }
    }

    #[test]
    fn writes_both_fans_in_the_driver_format() {
        let (dir, control) = control_with("0,0\n");
        let setting = FanSetting {
            cpu: FanSpeed::Percent(70),
            gpu: FanSpeed::Auto,
        };
        control.set(setting).unwrap();
        assert_eq!(
            read_trimmed(&dir.path().join(FAN_SPEED_FILE)).unwrap(),
            "70,0"
        );
    }

    #[test]
    fn what_it_writes_it_reads_back() {
        let (_dir, control) = control_with("0,0\n");
        let setting = FanSetting {
            cpu: FanSpeed::Percent(35),
            gpu: FanSpeed::Percent(100),
        };
        control.set(setting).unwrap();
        assert_eq!(control.get().unwrap(), setting);
    }

    #[test]
    fn fails_when_the_driver_is_not_loaded() {
        let dir = tempfile::tempdir().unwrap();
        let control = NitroSense::new(&dir.path().join("absent"));
        assert!(matches!(control.get(), Err(HardwareError::Io { .. })));
    }
}
