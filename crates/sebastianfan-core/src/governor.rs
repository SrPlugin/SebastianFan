use crate::curve::{Curve, Step};
use crate::profile::Profile;
use crate::temperature::Temperature;

#[derive(Debug, Clone)]
pub struct Governor {
    curve: Curve,
    hysteresis_celsius: u32,
    current: Step,
}

impl Governor {
    pub fn new(curve: Curve, hysteresis_celsius: u32) -> Self {
        let current = curve.lowest();
        Self {
            curve,
            hysteresis_celsius,
            current,
        }
    }

    pub fn profile(&self) -> Profile {
        self.current.profile
    }

    pub fn update(&mut self, temperature: Temperature) -> Profile {
        let target = self.curve.step_for(temperature);
        let rising = target.min_celsius > self.current.min_celsius;
        let falling = target.min_celsius < self.current.min_celsius
            && self.cooled_past_hysteresis(temperature);
        if rising || falling {
            self.current = target;
        }
        self.current.profile
    }

    fn cooled_past_hysteresis(&self, temperature: Temperature) -> bool {
        let release_celsius =
            i64::from(self.current.min_celsius) - i64::from(self.hysteresis_celsius);
        i64::from(temperature.millicelsius()) < release_celsius * 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn governor(hysteresis_celsius: u32) -> Governor {
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
        Governor::new(curve, hysteresis_celsius)
    }

    fn at(governor: &mut Governor, celsius: i32) -> Profile {
        governor.update(Temperature::from_celsius(celsius))
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
}
