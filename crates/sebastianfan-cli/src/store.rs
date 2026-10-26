use std::fs;
use std::io;
use std::path::Path;

use sebastianfan_core::Config;

use crate::error::CliError;

pub fn load(path: &Path) -> Result<Config, CliError> {
    match fs::read_to_string(path) {
        Ok(source) => Config::from_toml(&source).map_err(|source| CliError::InvalidConfig {
            path: path.to_path_buf(),
            source,
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
        Err(source) => Err(CliError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_yields_the_default_configuration() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load(&dir.path().join("absent.toml")).unwrap(),
            Config::default()
        );
    }

    #[test]
    fn loads_a_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "sensor = \"TCPU\"").unwrap();
        assert_eq!(load(&path).unwrap().sensor, "TCPU");
    }

    #[test]
    fn invalid_files_report_their_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "poll_interval_ms = 1").unwrap();
        let error = load(&path).unwrap_err();
        assert!(matches!(error, CliError::InvalidConfig { .. }));
        assert!(error.to_string().starts_with(&path.display().to_string()));
    }

    #[test]
    fn unreadable_paths_are_io_errors() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(load(dir.path()), Err(CliError::Io { .. })));
    }
}
