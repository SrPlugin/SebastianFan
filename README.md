# SebastianFan

Thermal profile control for Linux laptops, written in Rust.

SebastianFan reads your CPU temperature and switches the kernel's ACPI
`platform_profile` (`quiet`, `balanced`, `balanced-performance`, ...) along a
curve you define. The firmware applies its own fan curve and power limits for
each profile, so the fans follow the profile without any unsafe hardware access.

It ships as a git-style CLI and a systemd daemon.

> Developed on an Acer Nitro ANV15-51 running Linux 6.17. Any laptop that
> exposes `/sys/firmware/acpi/platform_profile` should work.

## Features

- Inspect thermal zones, fan speeds and the active profile
- Temperature curve with hysteresis, so profiles do not flap around a threshold
- Foreground or systemd-managed daemon that restores your original profile on exit
- Strict, validated TOML configuration
- No `unsafe` code and no runtime dependencies beyond the Linux kernel interfaces

## Requirements

- Linux with `platform_profile` support
- Rust 1.80 or newer to build
- Root privileges to change the profile or install the service

## Installation

```sh
git clone https://github.com/SrPlugin/SebastianFan.git
cd SebastianFan
cargo build --release
sudo install -m 0755 target/release/sebastianfan /usr/local/bin/sebastianfan
```

## Quick start

```sh
sebastianfan status
sudo sebastianfan config init
sudo sebastianfan daemon install
```

`daemon install` records the path of the running binary in the service file, so
run it from the installed location.

## Commands

The CLI follows a `sebastianfan <noun> <verb>` layout, like `git`.

```
sebastianfan status
sebastianfan sensor list
sebastianfan fan list
sebastianfan profile list
sebastianfan profile get
sebastianfan profile set <name>
sebastianfan curve show
sebastianfan curve validate <file>
sebastianfan config path
sebastianfan config show
sebastianfan config init [--force]
sebastianfan daemon run
sebastianfan daemon install
sebastianfan daemon uninstall
```

Global options:

| Option | Default | Description |
| --- | --- | --- |
| `--config <PATH>` | `/etc/sebastianfan/config.toml` | Configuration file |
| `--root <DIR>` | `/` | Filesystem root used to locate `/sys` and `/etc/systemd` |
| `--log-level <LEVEL>` | `info` | `error`, `warn`, `info`, `debug` or `trace` |

### Examples

```console
$ sebastianfan status
profile        balanced
temperature    46.0 °C (x86_pkg_temp)
fan acer/fan1  3546 rpm
fan acer/fan2  3307 rpm

$ sebastianfan sensor list
thermal_zone0  acpitz           42.0 °C
thermal_zone6  TCPU             42.0 °C
thermal_zone8  x86_pkg_temp     47.0 °C

$ sebastianfan profile list
  low-power
  quiet
* balanced
  balanced-performance

$ sudo sebastianfan profile set quiet
profile set to quiet

$ sebastianfan curve show
>= 0 °C   quiet
>= 55 °C  balanced
>= 75 °C  balanced-performance
```

## Configuration

`sebastianfan config init` writes the defaults to the configuration file. A
missing file means the defaults are used. Unknown keys are rejected.

```toml
sensor = "x86_pkg_temp"
poll_interval_ms = 1000
hysteresis_celsius = 5

[[curve]]
min_celsius = 0
profile = "quiet"

[[curve]]
min_celsius = 55
profile = "balanced"

[[curve]]
min_celsius = 75
profile = "balanced-performance"
```

| Key | Description |
| --- | --- |
| `sensor` | Thermal zone name (`thermal_zone8`) or type (`x86_pkg_temp`, `TCPU`), see `sensor list` |
| `poll_interval_ms` | Time between readings, at least 100 |
| `hysteresis_celsius` | Degrees the temperature must drop below a step before it steps down |
| `curve` | Steps ordered by strictly increasing `min_celsius` |

Check a file before installing it:

```sh
sebastianfan curve validate ./candidate.toml
```

## How it works

1. The daemon reads the configured thermal zone every `poll_interval_ms`.
2. The governor picks the highest curve step whose threshold has been reached.
3. It steps up immediately, and steps down only after the temperature falls
   below the step threshold minus `hysteresis_celsius`.
4. The profile is written to `platform_profile` only when it changes.
5. On `SIGINT` or `SIGTERM`, or if a reading fails, the original profile is
   restored before exiting.

Before starting, the daemon verifies that every profile in the curve is listed
in `platform_profile_choices` and refuses to run otherwise.

Profiles change the firmware fan curve and the power limits together, so a
`quiet` profile also lowers sustained performance.

## Limitations

SebastianFan selects profiles. It cannot set an exact fan speed: on the Acer
Nitro line that goes through an undocumented WMI method handled by the firmware.
A Rust kernel module for direct RPM control is planned.

## Project layout

```
crates/
  sebastianfan-core/     curve, governor, configuration and hardware traits
  sebastianfan-sysfs/    thermal zone, hwmon and platform profile adapters
  sebastianfan-daemon/   control loop and shutdown handling
  sebastianfan-cli/      the sebastianfan binary
dist/systemd/            service unit template
```

`core` has no knowledge of sysfs, and the daemon only depends on the
`SensorSource` and `ProfileControl` traits, so the loop is tested with fakes and
the adapters against temporary directory trees.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Commits follow [Conventional Commits](https://www.conventionalcommits.org/).

Use `--root` to run the CLI against a fake tree instead of the real `/sys`:

```sh
sebastianfan --root ./fixtures status
```

## License

MIT
