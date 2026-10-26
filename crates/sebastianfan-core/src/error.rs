use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unknown profile: {0}")]
    UnknownProfile(String),
    #[error("curve must contain at least one step")]
    EmptyCurve,
    #[error("curve steps must have strictly increasing min_celsius values")]
    UnorderedCurve,
}
