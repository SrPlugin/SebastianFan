use std::io;

use sebastianfan_core::{ConfigError, HardwareError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Hardware(#[from] HardwareError),
}

impl CliError {
    pub fn is_permission_denied(&self) -> bool {
        matches!(
            self,
            CliError::Hardware(HardwareError::Io { source, .. })
                if source.kind() == io::ErrorKind::PermissionDenied
        )
    }

    pub fn report(&self) -> String {
        if self.is_permission_denied() {
            format!("error: {self}\nhint: this operation usually requires root privileges")
        } else {
            format!("error: {self}")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn io_error(kind: io::ErrorKind) -> CliError {
        CliError::Hardware(HardwareError::Io {
            path: PathBuf::from("/sys/example"),
            source: io::Error::from(kind),
        })
    }

    #[test]
    fn permission_errors_suggest_root() {
        let report = io_error(io::ErrorKind::PermissionDenied).report();
        assert!(report.starts_with("error: /sys/example: "));
        assert!(report.contains("hint: "));
    }

    #[test]
    fn other_errors_have_no_hint() {
        assert!(!io_error(io::ErrorKind::NotFound).report().contains("hint"));
    }
}
