use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unknown profile: {0}")]
    UnknownProfile(String),
    #[error("curve must contain at least one step")]
    EmptyCurve,
    #[error("curve steps must have strictly increasing min_celsius values")]
    UnorderedCurve,
    #[error("sensor name must not be empty")]
    EmptySensor,
    #[error("poll_interval_ms must be at least {minimum_ms}")]
    PollIntervalTooShort { minimum_ms: u64 },
    #[error("invalid configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("cannot serialize configuration: {0}")]
    Serialize(#[from] toml::ser::Error),
}
