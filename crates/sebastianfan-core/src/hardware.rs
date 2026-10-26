use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::profile::Profile;
use crate::temperature::Temperature;

#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("{}: {source}", .path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{}: unexpected value {value:?}", .path.display())]
    Malformed { path: PathBuf, value: String },
    #[error("{0} not found")]
    NotFound(String),
    #[error("profile {0} is not supported by this machine")]
    UnsupportedProfile(Profile),
}

pub trait SensorSource {
    fn read(&self) -> Result<Temperature, HardwareError>;
}

pub trait ProfileControl {
    fn current(&self) -> Result<Profile, HardwareError>;
    fn available(&self) -> Result<Vec<Profile>, HardwareError>;
    fn set(&self, profile: Profile) -> Result<(), HardwareError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_errors_name_the_offending_path() {
        let error = HardwareError::Io {
            path: PathBuf::from("/sys/example"),
            source: io::Error::from(io::ErrorKind::PermissionDenied),
        };
        assert!(error.to_string().starts_with("/sys/example: "));
    }

    #[test]
    fn malformed_errors_quote_the_value() {
        let error = HardwareError::Malformed {
            path: PathBuf::from("/sys/example"),
            value: "abc".to_owned(),
        };
        assert_eq!(error.to_string(), "/sys/example: unexpected value \"abc\"");
    }

    #[test]
    fn unsupported_profile_names_the_profile() {
        let error = HardwareError::UnsupportedProfile(Profile::Performance);
        assert_eq!(
            error.to_string(),
            "profile performance is not supported by this machine"
        );
    }
}
