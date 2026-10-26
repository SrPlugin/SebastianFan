#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn fake_root() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "sys/class/thermal/thermal_zone0/type",
        "acpitz\n",
    );
    write(
        root.path(),
        "sys/class/thermal/thermal_zone0/temp",
        "42000\n",
    );
    write(
        root.path(),
        "sys/class/thermal/thermal_zone1/type",
        "x86_pkg_temp\n",
    );
    write(
        root.path(),
        "sys/class/thermal/thermal_zone1/temp",
        "43500\n",
    );
    root
}

fn sebastianfan(root: &TempDir) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sebastianfan"));
    command.arg("--root").arg(root.path());
    command
}

#[test]
fn prints_help_without_arguments() {
    Command::new(env!("CARGO_BIN_EXE_sebastianfan"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage: sebastianfan"));
}

#[test]
fn prints_the_version() {
    Command::new(env!("CARGO_BIN_EXE_sebastianfan"))
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("sebastianfan "));
}

#[test]
fn sensor_list_shows_every_zone_in_order() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["sensor", "list"])
        .assert()
        .success()
        .stdout("thermal_zone0  acpitz        42.0 °C\nthermal_zone1  x86_pkg_temp  43.5 °C\n");
}

#[test]
fn sensor_list_fails_when_sysfs_is_missing() {
    let empty = tempfile::tempdir().unwrap();
    sebastianfan(&empty)
        .args(["sensor", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: "));
}
