use std::fmt;
use std::str::FromStr;

use crate::error::ConfigError;
use crate::hardware::HardwareError;

pub const CHARGE_LIMIT_PERCENT: u8 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeLimit {
    Off,
    On,
}

impl ChargeLimit {
    pub const fn is_on(self) -> bool {
        matches!(self, Self::On)
    }
}

impl fmt::Display for ChargeLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => formatter.write_str("off"),
            Self::On => write!(formatter, "on ({CHARGE_LIMIT_PERCENT}%)"),
        }
    }
}

impl FromStr for ChargeLimit {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(ConfigError::InvalidSwitch(value.to_owned())),
        }
    }
}

pub trait BatteryControl {
    fn charge_limit(&self) -> Result<ChargeLimit, HardwareError>;
    fn set_charge_limit(&self, limit: ChargeLimit) -> Result<(), HardwareError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_on_and_off_in_any_case() {
        assert_eq!("on".parse::<ChargeLimit>().unwrap(), ChargeLimit::On);
        assert_eq!(" OFF ".parse::<ChargeLimit>().unwrap(), ChargeLimit::Off);
    }

    #[test]
    fn rejects_anything_else() {
        for value in ["", "1", "yes", "80", "true"] {
            let error = value.parse::<ChargeLimit>().unwrap_err();
            assert!(matches!(error, ConfigError::InvalidSwitch(_)), "{value}");
        }
    }

    #[test]
    fn the_error_explains_the_valid_values() {
        let message = "maybe".parse::<ChargeLimit>().unwrap_err().to_string();
        assert_eq!(message, "invalid value \"maybe\": use on or off");
    }

    #[test]
    fn displays_the_limit_percentage_when_on() {
        assert_eq!(ChargeLimit::On.to_string(), "on (80%)");
        assert_eq!(ChargeLimit::Off.to_string(), "off");
    }

    #[test]
    fn reports_whether_the_limit_is_active() {
        assert!(ChargeLimit::On.is_on());
        assert!(!ChargeLimit::Off.is_on());
    }
}
