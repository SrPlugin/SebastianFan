use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use sebastianfan_core::{Config, Governor, HardwareError, Profile, ProfileControl, SensorSource};

const SHUTDOWN_POLL: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct Daemon<S, C> {
    sensor: S,
    control: C,
    governor: Governor,
    interval: Duration,
    required: Vec<Profile>,
    applied: Option<Profile>,
}

impl<S: SensorSource, C: ProfileControl> Daemon<S, C> {
    pub fn new(sensor: S, control: C, config: &Config) -> Self {
        Self {
            sensor,
            control,
            governor: Governor::new(config.curve.clone(), config.hysteresis_celsius),
            interval: config.poll_interval(),
            required: config
                .curve
                .steps()
                .iter()
                .map(|step| step.profile)
                .collect(),
            applied: None,
        }
    }

    pub fn tick(&mut self) -> Result<Profile, HardwareError> {
        let temperature = self.sensor.read()?;
        let profile = self.governor.update(temperature);
        if self.applied != Some(profile) {
            self.control.set(profile)?;
            self.applied = Some(profile);
        }
        Ok(profile)
    }

    pub fn run(&mut self, shutdown: &AtomicBool) -> Result<(), HardwareError> {
        self.ensure_profiles_supported()?;
        let original = self.control.current()?;
        let outcome = self.run_until(shutdown);
        let restored = self.control.set(original);
        outcome.and(restored)
    }

    fn ensure_profiles_supported(&self) -> Result<(), HardwareError> {
        let available = self.control.available()?;
        match self
            .required
            .iter()
            .find(|profile| !available.contains(profile))
        {
            Some(profile) => Err(HardwareError::UnsupportedProfile(*profile)),
            None => Ok(()),
        }
    }

    fn run_until(&mut self, shutdown: &AtomicBool) -> Result<(), HardwareError> {
        while !shutdown.load(Ordering::Relaxed) {
            self.tick()?;
            self.wait(shutdown);
        }
        Ok(())
    }

    fn wait(&self, shutdown: &AtomicBool) {
        let deadline = Instant::now() + self.interval;
        while !shutdown.load(Ordering::Relaxed) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            thread::sleep(remaining.min(SHUTDOWN_POLL));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    use sebastianfan_core::{Config, Curve, Step, Temperature};

    use super::*;

    struct ScriptedSensor<'a> {
        readings: RefCell<VecDeque<i32>>,
        shutdown: &'a AtomicBool,
    }

    impl<'a> ScriptedSensor<'a> {
        fn new(celsius: &[i32], shutdown: &'a AtomicBool) -> Self {
            Self {
                readings: RefCell::new(celsius.iter().copied().collect()),
                shutdown,
            }
        }
    }

    impl SensorSource for ScriptedSensor<'_> {
        fn read(&self) -> Result<Temperature, HardwareError> {
            let mut readings = self.readings.borrow_mut();
            let next = readings
                .pop_front()
                .ok_or_else(|| HardwareError::NotFound("reading".into()))?;
            if readings.is_empty() {
                self.shutdown.store(true, Ordering::Relaxed);
            }
            Ok(Temperature::from_celsius(next))
        }
    }

    struct BrokenSensor;

    impl SensorSource for BrokenSensor {
        fn read(&self) -> Result<Temperature, HardwareError> {
            Err(HardwareError::NotFound("sensor".into()))
        }
    }

    #[derive(Clone)]
    struct RecordingControl {
        original: Profile,
        choices: Vec<Profile>,
        writes: Rc<RefCell<Vec<Profile>>>,
    }

    impl RecordingControl {
        fn new(original: Profile, choices: &[Profile]) -> Self {
            Self {
                original,
                choices: choices.to_vec(),
                writes: Rc::default(),
            }
        }

        fn writes(&self) -> Vec<Profile> {
            self.writes.borrow().clone()
        }
    }

    impl ProfileControl for RecordingControl {
        fn current(&self) -> Result<Profile, HardwareError> {
            Ok(self
                .writes
                .borrow()
                .last()
                .copied()
                .unwrap_or(self.original))
        }

        fn available(&self) -> Result<Vec<Profile>, HardwareError> {
            Ok(self.choices.clone())
        }

        fn set(&self, profile: Profile) -> Result<(), HardwareError> {
            self.writes.borrow_mut().push(profile);
            Ok(())
        }
    }

    fn config() -> Config {
        let curve = Curve::new(vec![
            Step {
                min_celsius: 0,
                profile: Profile::Quiet,
            },
            Step {
                min_celsius: 60,
                profile: Profile::Balanced,
            },
        ])
        .unwrap();
        Config {
            poll_interval_ms: 1,
            hysteresis_celsius: 5,
            curve,
            ..Config::default()
        }
    }

    fn all_profiles() -> [Profile; 3] {
        [
            Profile::Quiet,
            Profile::Balanced,
            Profile::BalancedPerformance,
        ]
    }

    #[test]
    fn tick_applies_the_profile_for_the_current_temperature() {
        let shutdown = AtomicBool::new(false);
        let control = RecordingControl::new(Profile::Balanced, &all_profiles());
        let mut daemon = Daemon::new(
            ScriptedSensor::new(&[70], &shutdown),
            control.clone(),
            &config(),
        );
        assert_eq!(daemon.tick().unwrap(), Profile::Balanced);
        assert_eq!(control.writes(), [Profile::Balanced]);
    }

    #[test]
    fn tick_does_not_rewrite_an_unchanged_profile() {
        let shutdown = AtomicBool::new(false);
        let control = RecordingControl::new(Profile::Balanced, &all_profiles());
        let mut daemon = Daemon::new(
            ScriptedSensor::new(&[30, 31, 32], &shutdown),
            control.clone(),
            &config(),
        );
        for _ in 0..3 {
            daemon.tick().unwrap();
        }
        assert_eq!(control.writes(), [Profile::Quiet]);
    }

    #[test]
    fn tick_follows_a_rising_temperature() {
        let shutdown = AtomicBool::new(false);
        let control = RecordingControl::new(Profile::Balanced, &all_profiles());
        let mut daemon = Daemon::new(
            ScriptedSensor::new(&[30, 65], &shutdown),
            control.clone(),
            &config(),
        );
        daemon.tick().unwrap();
        daemon.tick().unwrap();
        assert_eq!(control.writes(), [Profile::Quiet, Profile::Balanced]);
    }

    #[test]
    fn run_restores_the_original_profile_on_shutdown() {
        let shutdown = AtomicBool::new(false);
        let control = RecordingControl::new(Profile::BalancedPerformance, &all_profiles());
        let sensor = ScriptedSensor::new(&[30, 70], &shutdown);
        let mut daemon = Daemon::new(sensor, control.clone(), &config());
        daemon.run(&shutdown).unwrap();
        assert_eq!(
            control.writes(),
            [
                Profile::Quiet,
                Profile::Balanced,
                Profile::BalancedPerformance
            ]
        );
    }

    #[test]
    fn run_restores_the_original_profile_when_the_sensor_fails() {
        let shutdown = AtomicBool::new(false);
        let control = RecordingControl::new(Profile::Balanced, &all_profiles());
        let mut daemon = Daemon::new(BrokenSensor, control.clone(), &config());
        assert!(matches!(
            daemon.run(&shutdown),
            Err(HardwareError::NotFound(_))
        ));
        assert_eq!(control.writes(), [Profile::Balanced]);
    }

    #[test]
    fn run_refuses_a_curve_the_machine_cannot_apply() {
        let shutdown = AtomicBool::new(false);
        let control = RecordingControl::new(Profile::Quiet, &[Profile::Quiet]);
        let mut daemon = Daemon::new(BrokenSensor, control.clone(), &config());
        let result = daemon.run(&shutdown);
        assert!(matches!(
            result,
            Err(HardwareError::UnsupportedProfile(Profile::Balanced))
        ));
        assert!(control.writes().is_empty());
    }

    #[test]
    fn run_returns_immediately_when_shutdown_is_already_requested() {
        let shutdown = AtomicBool::new(true);
        let control = RecordingControl::new(Profile::Balanced, &all_profiles());
        let mut daemon = Daemon::new(BrokenSensor, control.clone(), &config());
        daemon.run(&shutdown).unwrap();
        assert_eq!(control.writes(), [Profile::Balanced]);
    }
}
