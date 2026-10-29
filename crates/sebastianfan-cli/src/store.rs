use std::fs;
use std::io;
use std::path::Path;

use sebastianfan_core::{Config, FanSetting};

use crate::error::CliError;

pub fn read(path: &Path) -> Result<Config, CliError> {
    let source = fs::read_to_string(path).map_err(|source| CliError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Config::from_toml(&source).map_err(|source| CliError::InvalidConfig {
        path: path.to_path_buf(),
        source,
    })
}

pub fn load(path: &Path) -> Result<Config, CliError> {
    match read(path) {
        Err(CliError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
            Ok(Config::default())
        }
        result => result,
    }
}

pub fn read_fans(path: &Path) -> Result<Option<FanSetting>, CliError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text.trim().parse()?)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(CliError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

pub fn write_fans(path: &Path, setting: FanSetting) -> Result<(), CliError> {
    let io_error = |source| CliError::Io {
        path: path.to_path_buf(),
        source,
    };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(io_error)?;
    }
    fs::write(path, format!("{},{}\n", setting.cpu, setting.gpu)).map_err(io_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sebastianfan_core::FanSpeed;

    #[test]
    fn a_missing_file_yields_the_default_configuration() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load(&dir.path().join("absent.toml")).unwrap(),
            Config::default()
        );
    }

    #[test]
    fn reading_a_missing_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            read(&dir.path().join("absent.toml")),
            Err(CliError::Io { .. })
        ));
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

    #[test]
    fn fan_settings_survive_a_write_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("var/lib/sebastianfan/fans");
        let setting = FanSetting {
            cpu: FanSpeed::Percent(70),
            gpu: FanSpeed::Auto,
        };
        write_fans(&path, setting).unwrap();
        assert_eq!(read_fans(&path).unwrap(), Some(setting));
    }

    #[test]
    fn no_saved_fan_setting_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_fans(&dir.path().join("absent")).unwrap(), None);
    }
}
