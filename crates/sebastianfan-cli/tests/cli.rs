#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;
use std::process::{Command as StdCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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
    write(root.path(), "sys/class/hwmon/hwmon3/name", "nvme\n");
    write(root.path(), "sys/class/hwmon/hwmon5/name", "acer\n");
    write(root.path(), "sys/class/hwmon/hwmon5/fan1_input", "3564\n");
    write(root.path(), "sys/class/hwmon/hwmon5/fan2_input", "3307\n");
    write(
        root.path(),
        "sys/firmware/acpi/platform_profile",
        "balanced\n",
    );
    write(
        root.path(),
        "sys/firmware/acpi/platform_profile_choices",
        "low-power quiet balanced balanced-performance\n",
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

#[test]
fn fan_list_shows_every_fan_with_its_speed() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["fan", "list"])
        .assert()
        .success()
        .stdout("acer  fan1  3564 rpm\nacer  fan2  3307 rpm\n");
}

#[test]
fn fan_list_is_empty_when_no_fan_is_exposed() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("sys/class/hwmon")).unwrap();
    sebastianfan(&root)
        .args(["fan", "list"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn fan_list_reports_unreadable_speeds_inline() {
    let root = fake_root();
    write(root.path(), "sys/class/hwmon/hwmon5/fan2_input", "fast\n");
    sebastianfan(&root)
        .args(["fan", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("unavailable (").and(predicate::str::contains("3564 rpm")),
        );
}

#[test]
fn profile_get_prints_the_active_profile() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["profile", "get"])
        .assert()
        .success()
        .stdout("balanced\n");
}

#[test]
fn profile_list_marks_the_active_profile() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["profile", "list"])
        .assert()
        .success()
        .stdout("  low-power\n  quiet\n* balanced\n  balanced-performance\n");
}

#[test]
fn profile_set_writes_the_profile() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["profile", "set", "quiet"])
        .assert()
        .success()
        .stdout("profile set to quiet\n");
    let written =
        fs::read_to_string(root.path().join("sys/firmware/acpi/platform_profile")).unwrap();
    assert_eq!(written, "quiet");
}

#[test]
fn profile_set_refuses_profiles_the_machine_does_not_offer() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["profile", "set", "performance"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not supported by this machine"));
}

#[test]
fn profile_set_rejects_unknown_names_before_touching_hardware() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["profile", "set", "turbo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("possible values"));
}

#[test]
fn status_summarizes_profile_temperature_and_fans() {
    let root = fake_root();
    let missing_config = root.path().join("absent.toml");
    sebastianfan(&root)
        .arg("--config")
        .arg(&missing_config)
        .arg("status")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("profile")
                .and(predicate::str::contains("balanced"))
                .and(predicate::str::contains("43.5 °C (x86_pkg_temp)"))
                .and(predicate::str::contains("fan acer/fan1"))
                .and(predicate::str::contains("3307 rpm")),
        );
}

#[test]
fn status_reads_the_sensor_from_the_configuration() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(&config, "sensor = \"acpitz\"").unwrap();
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("42.0 °C (acpitz)"));
}

#[test]
fn status_degrades_gracefully_on_unsupported_hardware() {
    let root = tempfile::tempdir().unwrap();
    let missing_config = root.path().join("absent.toml");
    sebastianfan(&root)
        .arg("--config")
        .arg(&missing_config)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("unavailable").count(3));
}

#[test]
fn status_fails_on_an_invalid_configuration() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(&config, "poll_interval_ms = 1").unwrap();
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "poll_interval_ms must be at least",
        ));
}

#[test]
fn curve_show_prints_the_default_curve() {
    let root = fake_root();
    sebastianfan(&root)
        .arg("--config")
        .arg(root.path().join("absent.toml"))
        .args(["curve", "show"])
        .assert()
        .success()
        .stdout(">= 0 °C   quiet\n>= 55 °C  balanced\n>= 75 °C  balanced-performance\n");
}

#[test]
fn curve_show_prints_the_configured_curve() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(
        &config,
        "[[curve]]\nmin_celsius = 40\nprofile = \"quiet\"\n",
    )
    .unwrap();
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .args(["curve", "show"])
        .assert()
        .success()
        .stdout(">= 40 °C  quiet\n");
}

#[test]
fn curve_validate_accepts_a_valid_file() {
    let root = fake_root();
    let file = root.path().join("candidate.toml");
    fs::write(&file, "hysteresis_celsius = 2").unwrap();
    sebastianfan(&root)
        .args(["curve", "validate"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::ends_with(": ok (3 steps)\n"));
}

#[test]
fn curve_validate_rejects_an_unordered_curve() {
    let root = fake_root();
    let file = root.path().join("candidate.toml");
    fs::write(
        &file,
        "[[curve]]\nmin_celsius = 60\nprofile = \"balanced\"\n\n\
         [[curve]]\nmin_celsius = 40\nprofile = \"quiet\"\n",
    )
    .unwrap();
    sebastianfan(&root)
        .args(["curve", "validate"])
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("strictly increasing"));
}

#[test]
fn curve_validate_fails_for_a_missing_file() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["curve", "validate"])
        .arg(root.path().join("absent.toml"))
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: "));
}

#[test]
fn config_path_prints_the_selected_file() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["--config", "/tmp/custom.toml", "config", "path"])
        .assert()
        .success()
        .stdout("/tmp/custom.toml\n");
}

#[test]
fn config_show_prints_the_defaults_when_no_file_exists() {
    let root = fake_root();
    sebastianfan(&root)
        .arg("--config")
        .arg(root.path().join("absent.toml"))
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("sensor = \"x86_pkg_temp\"")
                .and(predicate::str::contains("[[curve]]"))
                .and(predicate::str::ends_with("\"balanced-performance\"\n")),
        );
}

#[test]
fn config_init_writes_a_loadable_default_file() {
    let root = fake_root();
    let config = root.path().join("etc/sebastianfan/config.toml");
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .args(["config", "init"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("wrote "));
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .args(["curve", "validate"])
        .arg(&config)
        .assert()
        .success();
}

#[test]
fn config_init_refuses_to_overwrite_without_force() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(&config, "sensor = \"TCPU\"").unwrap();
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .args(["config", "init"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
    assert_eq!(fs::read_to_string(&config).unwrap(), "sensor = \"TCPU\"");
}

#[test]
fn config_init_overwrites_with_force() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(&config, "sensor = \"TCPU\"").unwrap();
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .args(["config", "init", "--force"])
        .assert()
        .success();
    assert!(fs::read_to_string(&config)
        .unwrap()
        .contains("x86_pkg_temp"));
}

fn wait_until(condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "condition not met within 5 seconds"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn spawn_daemon(root: &TempDir, config: &Path, extra: &[&str]) -> std::process::Child {
    StdCommand::new(env!("CARGO_BIN_EXE_sebastianfan"))
        .arg("--root")
        .arg(root.path())
        .arg("--config")
        .arg(config)
        .args(extra)
        .args(["daemon", "run"])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

fn terminate(child: &std::process::Child) {
    StdCommand::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
}

fn profile_file(root: &TempDir) -> std::path::PathBuf {
    root.path().join("sys/firmware/acpi/platform_profile")
}

#[test]
fn daemon_run_applies_the_curve_and_restores_the_profile_on_sigterm() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(&config, "poll_interval_ms = 100").unwrap();
    let child = spawn_daemon(&root, &config, &[]);
    wait_until(|| fs::read_to_string(profile_file(&root)).is_ok_and(|value| value == "quiet"));
    terminate(&child);
    let output = child.wait_with_output().unwrap();
    let log = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{log}");
    assert_eq!(fs::read_to_string(profile_file(&root)).unwrap(), "balanced");
    assert!(log.contains("profile changed"), "{log}");
    assert!(log.contains("restored original profile"), "{log}");
}

#[test]
fn daemon_run_honors_the_log_level() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(&config, "poll_interval_ms = 100").unwrap();
    let child = spawn_daemon(&root, &config, &["--log-level", "error"]);
    wait_until(|| fs::read_to_string(profile_file(&root)).is_ok_and(|value| value == "quiet"));
    terminate(&child);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn daemon_run_refuses_a_curve_the_machine_cannot_apply() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(
        &config,
        "[[curve]]\nmin_celsius = 0\nprofile = \"performance\"\n",
    )
    .unwrap();
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .args(["daemon", "run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not supported by this machine"));
    assert_eq!(
        fs::read_to_string(profile_file(&root)).unwrap(),
        "balanced\n"
    );
}

#[test]
fn daemon_run_fails_when_the_configured_sensor_does_not_exist() {
    let root = fake_root();
    let config = root.path().join("config.toml");
    fs::write(&config, "sensor = \"GPU\"").unwrap();
    sebastianfan(&root)
        .arg("--config")
        .arg(&config)
        .args(["daemon", "run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

fn unit_file(root: &TempDir) -> std::path::PathBuf {
    root.path().join("etc/systemd/system/sebastianfan.service")
}

#[test]
fn daemon_install_writes_the_unit_with_the_binary_and_config() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["--config", "/etc/custom/fan.toml", "daemon", "install"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("installed "));
    let unit = fs::read_to_string(unit_file(&root)).unwrap();
    let expected = format!(
        "ExecStart={} --config /etc/custom/fan.toml daemon run",
        env!("CARGO_BIN_EXE_sebastianfan")
    );
    assert!(unit.contains(&expected), "{unit}");
    assert!(!unit.contains('@'));
}

#[test]
fn daemon_uninstall_removes_the_unit() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["daemon", "install"])
        .assert()
        .success();
    sebastianfan(&root)
        .args(["daemon", "uninstall"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("removed "));
    assert!(!unit_file(&root).exists());
}

#[test]
fn daemon_uninstall_is_idempotent() {
    let root = fake_root();
    sebastianfan(&root)
        .args(["daemon", "uninstall"])
        .assert()
        .success()
        .stdout(predicate::str::ends_with("is not installed\n"));
}
