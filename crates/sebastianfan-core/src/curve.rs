use serde::{Deserialize, Serialize};

use crate::error::ConfigError;
use crate::profile::Profile;
use crate::temperature::Temperature;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub min_celsius: i32,
    pub profile: Profile,
}

impl Step {
    pub const fn threshold(self) -> Temperature {
        Temperature::from_celsius(self.min_celsius)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<Step>", into = "Vec<Step>")]
pub struct Curve {
    steps: Vec<Step>,
}

impl Curve {
    pub fn new(steps: Vec<Step>) -> Result<Self, ConfigError> {
        if steps.is_empty() {
            return Err(ConfigError::EmptyCurve);
        }
        if steps
            .windows(2)
            .any(|pair| pair[0].min_celsius >= pair[1].min_celsius)
        {
            return Err(ConfigError::UnorderedCurve);
        }
        Ok(Self { steps })
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    pub fn lowest(&self) -> Step {
        self.steps[0]
    }

    pub fn step_for(&self, temperature: Temperature) -> Step {
        self.steps
            .iter()
            .rev()
            .find(|step| temperature >= step.threshold())
            .copied()
            .unwrap_or_else(|| self.lowest())
    }
}

impl Default for Curve {
    fn default() -> Self {
        Self {
            steps: vec![
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
            ],
        }
    }
}

impl TryFrom<Vec<Step>> for Curve {
    type Error = ConfigError;

    fn try_from(steps: Vec<Step>) -> Result<Self, Self::Error> {
        Curve::new(steps)
    }
}

impl From<Curve> for Vec<Step> {
    fn from(curve: Curve) -> Self {
        curve.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(min_celsius: i32, profile: Profile) -> Step {
        Step {
            min_celsius,
            profile,
        }
    }

    fn sample() -> Curve {
        Curve::new(vec![
            step(0, Profile::Quiet),
            step(55, Profile::Balanced),
            step(75, Profile::BalancedPerformance),
        ])
        .unwrap()
    }

    #[test]
    fn rejects_an_empty_curve() {
        assert!(matches!(
            Curve::new(Vec::new()),
            Err(ConfigError::EmptyCurve)
        ));
    }

    #[test]
    fn rejects_decreasing_thresholds() {
        let steps = vec![step(60, Profile::Balanced), step(40, Profile::Quiet)];
        assert!(matches!(
            Curve::new(steps),
            Err(ConfigError::UnorderedCurve)
        ));
    }

    #[test]
    fn rejects_duplicate_thresholds() {
        let steps = vec![step(40, Profile::Quiet), step(40, Profile::Balanced)];
        assert!(matches!(
            Curve::new(steps),
            Err(ConfigError::UnorderedCurve)
        ));
    }

    #[test]
    fn selects_the_lowest_step_below_the_first_threshold() {
        let curve =
            Curve::new(vec![step(30, Profile::Quiet), step(60, Profile::Balanced)]).unwrap();
        assert_eq!(
            curve.step_for(Temperature::from_celsius(10)).profile,
            Profile::Quiet
        );
    }

    #[test]
    fn thresholds_are_inclusive() {
        let curve = sample();
        assert_eq!(
            curve.step_for(Temperature::from_celsius(55)).profile,
            Profile::Balanced
        );
        assert_eq!(
            curve
                .step_for(Temperature::from_millicelsius(54_999))
                .profile,
            Profile::Quiet
        );
    }

    #[test]
    fn selects_the_highest_step_above_every_threshold() {
        let profile = sample().step_for(Temperature::from_celsius(95)).profile;
        assert_eq!(profile, Profile::BalancedPerformance);
    }

    #[test]
    fn exposes_steps_in_order() {
        let minimums: Vec<i32> = sample()
            .steps()
            .iter()
            .map(|step| step.min_celsius)
            .collect();
        assert_eq!(minimums, [0, 55, 75]);
    }
}
