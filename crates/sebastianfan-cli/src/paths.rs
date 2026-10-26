use std::path::{Path, PathBuf};

use sebastianfan_sysfs::{HWMON_DIR, THERMAL_DIR};

#[derive(Debug, Clone)]
pub struct Paths {
    root: PathBuf,
}

impl Paths {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn thermal(&self) -> PathBuf {
        self.under(THERMAL_DIR)
    }

    pub fn hwmon(&self) -> PathBuf {
        self.under(HWMON_DIR)
    }

    fn under(&self, absolute: &str) -> PathBuf {
        self.root.join(absolute.trim_start_matches('/'))
    }
}

impl Default for Paths {
    fn default() -> Self {
        Self::new(Path::new("/").to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_root_keeps_system_paths() {
        assert_eq!(Paths::default().thermal(), Path::new("/sys/class/thermal"));
        assert_eq!(Paths::default().hwmon(), Path::new("/sys/class/hwmon"));
    }

    #[test]
    fn a_custom_root_prefixes_every_path() {
        let paths = Paths::new(PathBuf::from("/tmp/fake"));
        assert_eq!(paths.thermal(), Path::new("/tmp/fake/sys/class/thermal"));
        assert_eq!(paths.hwmon(), Path::new("/tmp/fake/sys/class/hwmon"));
    }
}
