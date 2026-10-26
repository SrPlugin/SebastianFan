mod battery;
mod config;
mod curve;
mod error;
mod fan;
mod governor;
mod hardware;
mod profile;
mod temperature;

pub use battery::{BatteryControl, ChargeLimit, CHARGE_LIMIT_PERCENT};
pub use config::{Config, MIN_POLL_INTERVAL_MS};
pub use curve::{Curve, Step};
pub use error::ConfigError;
pub use fan::{FanControl, FanSetting, FanSpeed, FanTarget, MAX_PERCENT, MIN_MANUAL_PERCENT};
pub use governor::Governor;
pub use hardware::{HardwareError, ProfileControl, SensorSource};
pub use profile::Profile;
pub use temperature::Temperature;
