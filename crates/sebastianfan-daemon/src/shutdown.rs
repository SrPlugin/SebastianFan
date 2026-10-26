use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;

pub fn shutdown_flag() -> io::Result<Arc<AtomicBool>> {
    let requested = Arc::new(AtomicBool::new(false));
    for signal in [SIGINT, SIGTERM] {
        flag::register(signal, Arc::clone(&requested))?;
    }
    Ok(requested)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use signal_hook::low_level;

    use super::*;

    #[test]
    fn starts_unset() {
        assert!(!shutdown_flag().unwrap().load(Ordering::Relaxed));
    }

    #[test]
    fn is_set_when_the_process_receives_sigterm() {
        let requested = shutdown_flag().unwrap();
        low_level::raise(SIGTERM).unwrap();
        assert!(requested.load(Ordering::Relaxed));
    }
}
