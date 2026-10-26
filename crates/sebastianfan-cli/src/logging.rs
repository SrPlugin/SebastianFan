use std::io;

use tracing::Level;

use crate::cli::LogLevel;

pub fn init(level: LogLevel) {
    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::from(level))
        .with_target(false)
        .without_time()
        .with_writer(io::stderr)
        .try_init();
}
