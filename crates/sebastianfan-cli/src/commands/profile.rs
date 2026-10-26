use sebastianfan_core::{Profile, ProfileControl};
use sebastianfan_sysfs::PlatformProfile;

use crate::error::CliError;
use crate::paths::Paths;

pub fn list(paths: &Paths) -> Result<String, CliError> {
    let control = PlatformProfile::new(&paths.platform_profile());
    let current = control.current()?;
    let lines: Vec<String> = control
        .available()?
        .iter()
        .map(|profile| {
            let marker = if *profile == current { '*' } else { ' ' };
            format!("{marker} {profile}")
        })
        .collect();
    Ok(lines.join("\n"))
}

pub fn get(paths: &Paths) -> Result<String, CliError> {
    let control = PlatformProfile::new(&paths.platform_profile());
    Ok(control.current()?.to_string())
}

pub fn set(paths: &Paths, profile: Profile) -> Result<String, CliError> {
    let control = PlatformProfile::new(&paths.platform_profile());
    control.set(profile)?;
    Ok(format!("profile set to {profile}"))
}
