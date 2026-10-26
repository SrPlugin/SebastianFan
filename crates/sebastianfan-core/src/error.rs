use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unknown profile: {0}")]
    UnknownProfile(String),
}
