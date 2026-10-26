use thiserror::Error;

use crate::fan::{MAX_PERCENT, MIN_MANUAL_PERCENT};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unknown profile: {0}")]
    UnknownProfile(String),
    #[error("curve must contain at least one step")]
    EmptyCurve,
    #[error("curve steps must have strictly increasing min_celsius values")]
    UnorderedCurve,
    #[error("invalid fan speed {0:?}: use auto or a percentage from {MIN_MANUAL_PERCENT} to {MAX_PERCENT}")]
    InvalidFanSpeed(String),
    #[error("invalid value {0:?}: use on or off")]
    InvalidSwitch(String),
    #[error("sensor name must not be empty")]
    EmptySensor,
    #[error("poll_interval_ms must be at least {minimum_ms}")]
    PollIntervalTooShort { minimum_ms: u64 },
    #[error("invalid configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("cannot serialize configuration: {0}")]
    Serialize(#[from] toml::ser::Error),
}
