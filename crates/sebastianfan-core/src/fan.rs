use std::fmt;
use std::str::FromStr;

use crate::error::ConfigError;
use crate::hardware::HardwareError;

pub const MIN_MANUAL_PERCENT: u8 = 20;
pub const MAX_PERCENT: u8 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanSpeed {
    Auto,
    Percent(u8),
}

impl FanSpeed {
    pub fn manual(percent: u8) -> Result<Self, ConfigError> {
        if (MIN_MANUAL_PERCENT..=MAX_PERCENT).contains(&percent) {
            Ok(Self::Percent(percent))
        } else {
            Err(ConfigError::InvalidFanSpeed(percent.to_string()))
        }
    }

    pub const fn is_manual(self) -> bool {
        matches!(self, Self::Percent(_))
    }
}

impl fmt::Display for FanSpeed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => formatter.write_str("auto"),
            Self::Percent(percent) => write!(formatter, "{percent}%"),
        }
    }
}

impl FromStr for FanSpeed {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = value.trim();
        if trimmed.eq_ignore_ascii_case("auto") {
            return Ok(Self::Auto);
        }
        trimmed
            .strip_suffix('%')
            .unwrap_or(trimmed)
            .parse::<u8>()
            .ok()
            .and_then(|percent| Self::manual(percent).ok())
            .ok_or_else(|| ConfigError::InvalidFanSpeed(value.to_owned()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanTarget {
    Cpu,
    Gpu,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanSetting {
    pub cpu: FanSpeed,
    pub gpu: FanSpeed,
}

impl FanSetting {
    pub const AUTO: Self = Self {
        cpu: FanSpeed::Auto,
        gpu: FanSpeed::Auto,
    };

    pub const fn is_manual(self) -> bool {
        self.cpu.is_manual() || self.gpu.is_manual()
    }

    #[must_use]
    pub const fn with(self, target: FanTarget, speed: FanSpeed) -> Self {
        match target {
            FanTarget::Cpu => Self { cpu: speed, ..self },
            FanTarget::Gpu => Self { gpu: speed, ..self },
            FanTarget::All => Self {
                cpu: speed,
                gpu: speed,
            },
        }
    }
}

impl fmt::Display for FanSetting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cpu {}, gpu {}", self.cpu, self.gpu)
    }
}

impl FromStr for FanSetting {
    type Err = ConfigError;

    /// Parses the `cpu,gpu` form, e.g. `70%,auto`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (cpu, gpu) = value
            .split_once(',')
            .ok_or_else(|| ConfigError::InvalidFanSpeed(value.to_owned()))?;
        Ok(Self {
            cpu: cpu.parse()?,
            gpu: gpu.parse()?,
        })
    }
}

pub trait FanControl {
    fn get(&self) -> Result<FanSetting, HardwareError>;
    fn set(&self, setting: FanSetting) -> Result<(), HardwareError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: &str) -> Result<FanSpeed, ConfigError> {
        value.parse()
    }

    #[test]
    fn parses_auto_in_any_case() {
        assert_eq!(parse("auto").unwrap(), FanSpeed::Auto);
        assert_eq!(parse(" AUTO ").unwrap(), FanSpeed::Auto);
    }

    #[test]
    fn parses_percentages_with_or_without_the_sign() {
        assert_eq!(parse("60").unwrap(), FanSpeed::Percent(60));
        assert_eq!(parse("60%").unwrap(), FanSpeed::Percent(60));
    }

    #[test]
    fn accepts_the_range_boundaries() {
        assert_eq!(parse("20").unwrap(), FanSpeed::Percent(20));
        assert_eq!(parse("100").unwrap(), FanSpeed::Percent(100));
    }

    #[test]
    fn rejects_values_outside_the_manual_range() {
        for value in ["0", "19", "101", "255", "-5", "", "fast", "60.5"] {
            assert!(
                matches!(parse(value), Err(ConfigError::InvalidFanSpeed(_))),
                "{value}"
            );
        }
    }

    #[test]
    fn the_error_explains_the_valid_range() {
        let message = parse("5").unwrap_err().to_string();
        assert_eq!(
            message,
            "invalid fan speed \"5\": use auto or a percentage from 20 to 100"
        );
    }

    #[test]
    fn parses_a_setting_from_the_cpu_gpu_form() {
        let setting: FanSetting = "70%,auto".parse().unwrap();
        assert_eq!(setting.cpu, FanSpeed::Percent(70));
        assert_eq!(setting.gpu, FanSpeed::Auto);
        assert!("70".parse::<FanSetting>().is_err());
        assert!("70,5".parse::<FanSetting>().is_err());
    }

    #[test]
    fn displays_speeds_and_settings() {
        let setting = FanSetting {
            cpu: FanSpeed::Percent(60),
            gpu: FanSpeed::Auto,
        };
        assert_eq!(setting.to_string(), "cpu 60%, gpu auto");
    }

    #[test]
    fn applies_a_speed_to_one_fan_and_keeps_the_other() {
        let base = FanSetting {
            cpu: FanSpeed::Auto,
            gpu: FanSpeed::Percent(40),
        };
        let updated = base.with(FanTarget::Cpu, FanSpeed::Percent(70));
        assert_eq!(
            updated,
            FanSetting {
                cpu: FanSpeed::Percent(70),
                gpu: FanSpeed::Percent(40)
            }
        );
    }

    #[test]
    fn applies_a_speed_to_both_fans() {
        let updated = FanSetting::AUTO.with(FanTarget::All, FanSpeed::Percent(50));
        assert_eq!(updated.cpu, updated.gpu);
        assert_eq!(updated.cpu, FanSpeed::Percent(50));
    }

    #[test]
    fn detects_manual_control() {
        assert!(!FanSetting::AUTO.is_manual());
        assert!(FanSetting::AUTO
            .with(FanTarget::Gpu, FanSpeed::Percent(30))
            .is_manual());
    }
}
