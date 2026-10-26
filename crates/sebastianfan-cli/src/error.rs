use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

use sebastianfan_core::{ConfigError, HardwareError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Hardware(#[from] HardwareError),
    #[error("{}: {source}", .path.display())]
    InvalidConfig {
        path: PathBuf,
        #[source]
        source: ConfigError,
    },
    #[error("cannot install signal handlers: {0}")]
    Signals(#[source] io::Error),
    #[error("systemctl {command} failed ({status})")]
    Systemctl { command: String, status: ExitStatus },
    #[error("{} already exists, use --force to overwrite it", .0.display())]
    AlreadyExists(PathBuf),
    #[error("{}: {source}", .path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl CliError {
    pub fn is_permission_denied(&self) -> bool {
        match self {
            CliError::Hardware(HardwareError::Io { source, .. }) | CliError::Io { source, .. } => {
                source.kind() == io::ErrorKind::PermissionDenied
            }
            _ => false,
        }
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
    fn permission_errors_on_plain_files_suggest_root() {
        let error = CliError::Io {
            path: PathBuf::from("/etc/sebastianfan/config.toml"),
            source: io::Error::from(io::ErrorKind::PermissionDenied),
        };
        assert!(error.report().contains("hint: "));
    }

    #[test]
    fn other_errors_have_no_hint() {
        assert!(!io_error(io::ErrorKind::NotFound).report().contains("hint"));
    }
}
