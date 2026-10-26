use std::path::{Path, PathBuf};

use sebastianfan_core::{HardwareError, Profile, ProfileControl};

use crate::file::{read_trimmed, write_value};

pub const PLATFORM_PROFILE_DIR: &str = "/sys/firmware/acpi";

const PROFILE_FILE: &str = "platform_profile";
const CHOICES_FILE: &str = "platform_profile_choices";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformProfile {
    profile_path: PathBuf,
    choices_path: PathBuf,
}

impl PlatformProfile {
    pub fn new(dir: &Path) -> Self {
        Self {
            profile_path: dir.join(PROFILE_FILE),
            choices_path: dir.join(CHOICES_FILE),
        }
    }

    pub fn system() -> Self {
        Self::new(Path::new(PLATFORM_PROFILE_DIR))
    }
}

impl ProfileControl for PlatformProfile {
    fn current(&self) -> Result<Profile, HardwareError> {
        let value = read_trimmed(&self.profile_path)?;
        value.parse().map_err(|_| HardwareError::Malformed {
            path: self.profile_path.clone(),
            value,
        })
    }

    fn available(&self) -> Result<Vec<Profile>, HardwareError> {
        let choices = read_trimmed(&self.choices_path)?;
        Ok(choices
            .split_whitespace()
            .filter_map(|name| name.parse().ok())
            .collect())
    }

    fn set(&self, profile: Profile) -> Result<(), HardwareError> {
        if !self.available()?.contains(&profile) {
            return Err(HardwareError::UnsupportedProfile(profile));
        }
        write_value(&self.profile_path, profile.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::test_support::write_file;

    fn fake_sysfs() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), PROFILE_FILE, "balanced\n");
        write_file(
            dir.path(),
            CHOICES_FILE,
            "low-power quiet balanced balanced-performance\n",
        );
        dir
    }

    #[test]
    fn reads_the_current_profile() {
        let dir = fake_sysfs();
        assert_eq!(
            PlatformProfile::new(dir.path()).current().unwrap(),
            Profile::Balanced
        );
    }

    #[test]
    fn lists_the_available_profiles() {
        let dir = fake_sysfs();
        let available = PlatformProfile::new(dir.path()).available().unwrap();
        assert_eq!(
            available,
            [
                Profile::LowPower,
                Profile::Quiet,
                Profile::Balanced,
                Profile::BalancedPerformance
            ]
        );
    }

    #[test]
    fn ignores_choices_it_does_not_know() {
        let dir = fake_sysfs();
        write_file(dir.path(), CHOICES_FILE, "quiet ludicrous balanced\n");
        let available = PlatformProfile::new(dir.path()).available().unwrap();
        assert_eq!(available, [Profile::Quiet, Profile::Balanced]);
    }

    #[test]
    fn writes_a_supported_profile() {
        let dir = fake_sysfs();
        let control = PlatformProfile::new(dir.path());
        control.set(Profile::Quiet).unwrap();
        assert_eq!(control.current().unwrap(), Profile::Quiet);
    }

    #[test]
    fn refuses_an_unsupported_profile_without_writing() {
        let dir = fake_sysfs();
        let control = PlatformProfile::new(dir.path());
        let error = control.set(Profile::Performance).unwrap_err();
        assert!(matches!(
            error,
            HardwareError::UnsupportedProfile(Profile::Performance)
        ));
        assert_eq!(control.current().unwrap(), Profile::Balanced);
    }

    #[test]
    fn reports_an_unrecognized_current_profile() {
        let dir = fake_sysfs();
        write_file(dir.path(), PROFILE_FILE, "ludicrous\n");
        let error = PlatformProfile::new(dir.path()).current().unwrap_err();
        assert!(matches!(error, HardwareError::Malformed { value, .. } if value == "ludicrous"));
    }

    #[test]
    fn fails_when_the_platform_does_not_expose_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let error = PlatformProfile::new(dir.path()).current().unwrap_err();
        assert!(matches!(error, HardwareError::Io { .. }));
    }
}
