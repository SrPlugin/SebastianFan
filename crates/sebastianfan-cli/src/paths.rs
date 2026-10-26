use std::path::{Path, PathBuf};

use sebastianfan_sysfs::{HWMON_DIR, NITRO_SENSE_DIR, PLATFORM_PROFILE_DIR, THERMAL_DIR};

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

    pub fn platform_profile(&self) -> PathBuf {
        self.under(PLATFORM_PROFILE_DIR)
    }

    pub fn nitro_sense(&self) -> PathBuf {
        self.under(NITRO_SENSE_DIR)
    }

    pub fn systemd_units(&self) -> PathBuf {
        self.under("/etc/systemd/system")
    }

    pub fn is_system(&self) -> bool {
        self.root == Path::new("/")
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
        assert_eq!(
            Paths::default().nitro_sense(),
            Path::new("/sys/devices/platform/acer-wmi/nitro_sense")
        );
        assert_eq!(
            Paths::default().systemd_units(),
            Path::new("/etc/systemd/system")
        );
        assert!(Paths::default().is_system());
        assert_eq!(
            Paths::default().platform_profile(),
            Path::new("/sys/firmware/acpi")
        );
    }

    #[test]
    fn a_custom_root_prefixes_every_path() {
        let paths = Paths::new(PathBuf::from("/tmp/fake"));
        assert_eq!(paths.thermal(), Path::new("/tmp/fake/sys/class/thermal"));
        assert_eq!(paths.hwmon(), Path::new("/tmp/fake/sys/class/hwmon"));
        assert_eq!(
            paths.systemd_units(),
            Path::new("/tmp/fake/etc/systemd/system")
        );
        assert!(!paths.is_system());
        assert_eq!(
            paths.platform_profile(),
            Path::new("/tmp/fake/sys/firmware/acpi")
        );
    }
}
