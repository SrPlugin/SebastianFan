use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::CliError;
use crate::paths::Paths;

const UNIT_NAME: &str = "sebastianfan.service";
const UNIT_TEMPLATE: &str = include_str!("../../../../dist/systemd/sebastianfan.service");

fn render_unit(binary: &Path, config: &Path) -> String {
    UNIT_TEMPLATE
        .replace("@BINARY@", &binary.display().to_string())
        .replace("@CONFIG@", &config.display().to_string())
}

fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> CliError + '_ {
    move |source| CliError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn systemctl(args: &[&str]) -> Result<(), CliError> {
    let status = Command::new("systemctl")
        .args(args)
        .status()
        .map_err(io_error(Path::new("systemctl")))?;
    if status.success() {
        Ok(())
    } else {
        Err(CliError::Systemctl {
            command: args.join(" "),
            status,
        })
    }
}

pub fn install(paths: &Paths, config: &Path) -> Result<String, CliError> {
    let binary = env::current_exe().map_err(io_error(Path::new("current executable")))?;
    let unit_path = paths.systemd_units().join(UNIT_NAME);
    if let Some(directory) = unit_path.parent() {
        fs::create_dir_all(directory).map_err(io_error(directory))?;
    }
    fs::write(&unit_path, render_unit(&binary, config)).map_err(io_error(&unit_path))?;
    if paths.is_system() {
        systemctl(&["daemon-reload"])?;
        systemctl(&["enable", "--now", UNIT_NAME])?;
    }
    Ok(format!("installed {}", unit_path.display()))
}

pub fn uninstall(paths: &Paths) -> Result<String, CliError> {
    let unit_path: PathBuf = paths.systemd_units().join(UNIT_NAME);
    if !unit_path.exists() {
        return Ok(format!("{UNIT_NAME} is not installed"));
    }
    if paths.is_system() {
        systemctl(&["disable", "--now", UNIT_NAME])?;
    }
    fs::remove_file(&unit_path).map_err(io_error(&unit_path))?;
    if paths.is_system() {
        systemctl(&["daemon-reload"])?;
    }
    Ok(format!("removed {}", unit_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_in_every_placeholder() {
        let unit = render_unit(
            Path::new("/usr/local/bin/sebastianfan"),
            Path::new("/etc/x.toml"),
        );
        assert!(
            unit.contains("ExecStart=/usr/local/bin/sebastianfan --config /etc/x.toml daemon run")
        );
        assert!(!unit.contains('@'));
    }

    #[test]
    fn the_unit_restarts_on_failure_and_stops_on_sigterm() {
        let unit = render_unit(Path::new("/bin/x"), Path::new("/etc/x.toml"));
        assert!(unit.contains("Restart=on-failure"));
        assert!(unit.contains("KillSignal=SIGTERM"));
    }
}
