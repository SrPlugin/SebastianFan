use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use sebastianfan_core::HardwareError;

fn io_error(path: &Path, source: std::io::Error) -> HardwareError {
    HardwareError::Io {
        path: path.to_path_buf(),
        source,
    }
}

pub(crate) fn read_trimmed(path: &Path) -> Result<String, HardwareError> {
    fs::read_to_string(path)
        .map(|content| content.trim().to_owned())
        .map_err(|source| io_error(path, source))
}

pub(crate) fn read_parsed<T: FromStr>(path: &Path) -> Result<T, HardwareError> {
    let value = read_trimmed(path)?;
    value.parse().map_err(|_| HardwareError::Malformed {
        path: path.to_path_buf(),
        value,
    })
}

pub(crate) fn write_value(path: &Path, value: &str) -> Result<(), HardwareError> {
    fs::write(path, value).map_err(|source| io_error(path, source))
}

pub(crate) fn list_entries(dir: &Path) -> Result<Vec<PathBuf>, HardwareError> {
    let entries = fs::read_dir(dir).map_err(|source| io_error(dir, source))?;
    Ok(entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect())
}

pub(crate) fn numbered_suffix(path: &Path, prefix: &str) -> Option<u32> {
    path.file_name()?
        .to_str()?
        .strip_prefix(prefix)?
        .parse()
        .ok()
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::fs;
    use std::path::Path;

    pub(crate) fn write_file(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::write_file;
    use super::*;

    #[test]
    fn trims_trailing_newlines() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "value", "42\n");
        assert_eq!(read_trimmed(&dir.path().join("value")).unwrap(), "42");
    }

    #[test]
    fn reports_missing_files_with_their_path() {
        let dir = tempfile::tempdir().unwrap();
        let error = read_trimmed(&dir.path().join("missing")).unwrap_err();
        assert!(matches!(error, HardwareError::Io { .. }));
    }

    #[test]
    fn reports_unparsable_values_as_malformed() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "value", "abc");
        let error = read_parsed::<u32>(&dir.path().join("value")).unwrap_err();
        assert!(matches!(error, HardwareError::Malformed { value, .. } if value == "abc"));
    }

    #[test]
    fn writes_values_to_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "value", "old");
        write_value(&dir.path().join("value"), "new").unwrap();
        assert_eq!(read_trimmed(&dir.path().join("value")).unwrap(), "new");
    }

    #[test]
    fn extracts_numbered_suffixes() {
        assert_eq!(
            numbered_suffix(Path::new("/x/thermal_zone12"), "thermal_zone"),
            Some(12)
        );
        assert_eq!(
            numbered_suffix(Path::new("/x/cooling_device1"), "thermal_zone"),
            None
        );
        assert_eq!(
            numbered_suffix(Path::new("/x/thermal_zoneX"), "thermal_zone"),
            None
        );
    }
}
