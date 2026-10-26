use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::curve::Curve;
use crate::error::ConfigError;

pub const MIN_POLL_INTERVAL_MS: u64 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub sensor: String,
    pub poll_interval_ms: u64,
    pub hysteresis_celsius: u32,
    pub min_hold_seconds: u64,
    pub curve: Curve,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sensor: "x86_pkg_temp".to_owned(),
            poll_interval_ms: 1000,
            hysteresis_celsius: 5,
            min_hold_seconds: 30,
            curve: Curve::default(),
        }
    }
}

impl Config {
    pub fn from_toml(source: &str) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(source)?;
        config.validate()?;
        Ok(config)
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        Ok(toml::to_string_pretty(self)?)
    }

    pub fn min_hold(&self) -> Duration {
        Duration::from_secs(self.min_hold_seconds)
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.sensor.trim().is_empty() {
            return Err(ConfigError::EmptySensor);
        }
        if self.poll_interval_ms < MIN_POLL_INTERVAL_MS {
            return Err(ConfigError::PollIntervalTooShort {
                minimum_ms: MIN_POLL_INTERVAL_MS,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;

    #[test]
    fn default_configuration_round_trips_through_toml() {
        let config = Config::default();
        let reparsed = Config::from_toml(&config.to_toml().unwrap()).unwrap();
        assert_eq!(reparsed, config);
    }

    #[test]
    fn empty_document_yields_defaults() {
        assert_eq!(Config::from_toml("").unwrap(), Config::default());
    }

    #[test]
    fn parses_a_complete_document() {
        let source = r#"
            sensor = "TCPU"
            poll_interval_ms = 500
            hysteresis_celsius = 3
            min_hold_seconds = 12

            [[curve]]
            min_celsius = 20
            profile = "quiet"

            [[curve]]
            min_celsius = 70
            profile = "performance"
        "#;
        let config = Config::from_toml(source).unwrap();
        assert_eq!(config.sensor, "TCPU");
        assert_eq!(config.poll_interval(), Duration::from_millis(500));
        assert_eq!(config.hysteresis_celsius, 3);
        assert_eq!(config.min_hold(), Duration::from_secs(12));
        assert_eq!(config.curve.steps().len(), 2);
        assert_eq!(config.curve.steps()[1].profile, Profile::Performance);
    }

    #[test]
    fn omitted_fields_keep_their_defaults() {
        let config = Config::from_toml("hysteresis_celsius = 8").unwrap();
        assert_eq!(config.hysteresis_celsius, 8);
        assert_eq!(config.sensor, Config::default().sensor);
    }

    #[test]
    fn the_default_hold_is_thirty_seconds() {
        assert_eq!(Config::default().min_hold(), Duration::from_secs(30));
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(matches!(
            Config::from_toml("fan_speed = 3"),
            Err(ConfigError::Parse(_))
        ));
    }

    #[test]
    fn rejects_unknown_profiles() {
        let source = "[[curve]]\nmin_celsius = 0\nprofile = \"turbo\"";
        assert!(matches!(
            Config::from_toml(source),
            Err(ConfigError::Parse(_))
        ));
    }

    #[test]
    fn rejects_an_unordered_curve() {
        let source = "[[curve]]\nmin_celsius = 60\nprofile = \"balanced\"\n\n\
                      [[curve]]\nmin_celsius = 40\nprofile = \"quiet\"";
        assert!(matches!(
            Config::from_toml(source),
            Err(ConfigError::Parse(_))
        ));
    }

    #[test]
    fn rejects_an_empty_curve() {
        assert!(matches!(
            Config::from_toml("curve = []"),
            Err(ConfigError::Parse(_))
        ));
    }

    #[test]
    fn rejects_a_blank_sensor() {
        assert!(matches!(
            Config::from_toml("sensor = \"  \""),
            Err(ConfigError::EmptySensor)
        ));
    }

    #[test]
    fn rejects_a_poll_interval_below_the_minimum() {
        let result = Config::from_toml("poll_interval_ms = 10");
        assert!(matches!(
            result,
            Err(ConfigError::PollIntervalTooShort { minimum_ms: 100 })
        ));
    }
}
