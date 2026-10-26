use std::time::{Duration, Instant};

use crate::curve::{Curve, Step};
use crate::profile::Profile;
use crate::temperature::Temperature;

#[derive(Debug, Clone)]
pub struct Governor {
    curve: Curve,
    hysteresis_celsius: u32,
    hold: Duration,
    current: Step,
    changed_at: Option<Instant>,
}

impl Governor {
    pub fn new(curve: Curve, hysteresis_celsius: u32, hold: Duration) -> Self {
        let current = curve.lowest();
        Self {
            curve,
            hysteresis_celsius,
            hold,
            current,
            changed_at: None,
        }
    }

    pub fn profile(&self) -> Profile {
        self.current.profile
    }

    pub fn update(&mut self, temperature: Temperature, now: Instant) -> Profile {
        let target = self.curve.step_for(temperature);
        let rising = target.min_celsius > self.current.min_celsius;
        let falling = target.min_celsius < self.current.min_celsius
            && self.cooled_past_hysteresis(temperature)
            && self.held_long_enough(now);
        if rising || falling {
            self.current = target;
            self.changed_at = Some(now);
        }
        self.current.profile
    }

    fn cooled_past_hysteresis(&self, temperature: Temperature) -> bool {
        let release_celsius =
            i64::from(self.current.min_celsius) - i64::from(self.hysteresis_celsius);
        i64::from(temperature.millicelsius()) < release_celsius * 1000
    }

    fn held_long_enough(&self, now: Instant) -> bool {
        self.changed_at
            .is_none_or(|changed| now.saturating_duration_since(changed) >= self.hold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn governor_with_hold(hysteresis_celsius: u32, hold: Duration) -> Governor {
        let curve = Curve::new(vec![
            Step {
                min_celsius: 0,
                profile: Profile::Quiet,
            },
            Step {
                min_celsius: 55,
                profile: Profile::Balanced,
            },
            Step {
                min_celsius: 75,
                profile: Profile::BalancedPerformance,
            },
        ])
        .unwrap();
        Governor::new(curve, hysteresis_celsius, hold)
    }

    fn governor(hysteresis_celsius: u32) -> Governor {
        governor_with_hold(hysteresis_celsius, Duration::ZERO)
    }

    fn at(governor: &mut Governor, celsius: i32) -> Profile {
        governor.update(Temperature::from_celsius(celsius), Instant::now())
    }

    fn after(governor: &mut Governor, start: Instant, seconds: u64, celsius: i32) -> Profile {
        governor.update(
            Temperature::from_celsius(celsius),
            start + Duration::from_secs(seconds),
        )
    }

    #[test]
    fn starts_on_the_lowest_step() {
        assert_eq!(governor(5).profile(), Profile::Quiet);
    }

    #[test]
    fn rises_as_soon_as_a_threshold_is_reached() {
        let mut governor = governor(5);
        assert_eq!(at(&mut governor, 54), Profile::Quiet);
        assert_eq!(at(&mut governor, 55), Profile::Balanced);
    }

    #[test]
    fn can_skip_levels_when_heating_quickly() {
        assert_eq!(at(&mut governor(5), 90), Profile::BalancedPerformance);
    }

    #[test]
    fn holds_the_profile_inside_the_hysteresis_band() {
        let mut governor = governor(5);
        at(&mut governor, 60);
        assert_eq!(at(&mut governor, 50), Profile::Balanced);
    }

    #[test]
    fn falls_once_below_the_hysteresis_band() {
        let mut governor = governor(5);
        at(&mut governor, 60);
        assert_eq!(at(&mut governor, 49), Profile::Quiet);
    }

    #[test]
    fn falls_through_several_levels_when_cooling_quickly() {
        let mut governor = governor(5);
        at(&mut governor, 90);
        assert_eq!(at(&mut governor, 20), Profile::Quiet);
    }

    #[test]
    fn zero_hysteresis_follows_the_curve_exactly() {
        let mut governor = governor(0);
        at(&mut governor, 60);
        assert_eq!(at(&mut governor, 54), Profile::Quiet);
    }

    #[test]
    fn does_not_flap_around_a_threshold() {
        let mut governor = governor(5);
        let readings = [56, 54, 56, 53, 57, 52];
        let profiles: Vec<Profile> = readings
            .iter()
            .map(|celsius| at(&mut governor, *celsius))
            .collect();
        assert!(profiles.iter().all(|profile| *profile == Profile::Balanced));
    }

    #[test]
    fn waits_for_the_hold_time_before_stepping_down() {
        let start = Instant::now();
        let mut governor = governor_with_hold(5, Duration::from_secs(30));
        assert_eq!(after(&mut governor, start, 0, 60), Profile::Balanced);
        assert_eq!(after(&mut governor, start, 10, 20), Profile::Balanced);
        assert_eq!(after(&mut governor, start, 29, 20), Profile::Balanced);
        assert_eq!(after(&mut governor, start, 30, 20), Profile::Quiet);
    }

    #[test]
    fn the_hold_never_delays_a_rise() {
        let start = Instant::now();
        let mut governor = governor_with_hold(5, Duration::from_secs(30));
        after(&mut governor, start, 0, 60);
        assert_eq!(
            after(&mut governor, start, 1, 80),
            Profile::BalancedPerformance
        );
    }

    #[test]
    fn the_hold_restarts_after_every_change() {
        let start = Instant::now();
        let mut governor = governor_with_hold(5, Duration::from_secs(30));
        after(&mut governor, start, 0, 60);
        after(&mut governor, start, 30, 20);
        after(&mut governor, start, 31, 60);
        assert_eq!(after(&mut governor, start, 40, 20), Profile::Balanced);
        assert_eq!(after(&mut governor, start, 61, 20), Profile::Quiet);
    }

    #[test]
    fn a_spiky_reading_does_not_bounce_the_profile() {
        let start = Instant::now();
        let mut governor = governor_with_hold(5, Duration::from_secs(30));
        let readings = [(0, 48), (3, 56), (6, 48), (9, 57), (12, 48), (15, 56)];
        let profiles: Vec<Profile> = readings
            .iter()
            .map(|(seconds, celsius)| after(&mut governor, start, *seconds, *celsius))
            .collect();
        assert_eq!(profiles[0], Profile::Quiet);
        assert!(profiles[1..]
            .iter()
            .all(|profile| *profile == Profile::Balanced));
    }
}
