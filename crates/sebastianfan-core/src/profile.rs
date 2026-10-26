use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    LowPower,
    Cool,
    Quiet,
    Balanced,
    BalancedPerformance,
    Performance,
    Custom,
}

impl Profile {
    pub const ALL: [Profile; 7] = [
        Profile::LowPower,
        Profile::Cool,
        Profile::Quiet,
        Profile::Balanced,
        Profile::BalancedPerformance,
        Profile::Performance,
        Profile::Custom,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Profile::LowPower => "low-power",
            Profile::Cool => "cool",
            Profile::Quiet => "quiet",
            Profile::Balanced => "balanced",
            Profile::BalancedPerformance => "balanced-performance",
            Profile::Performance => "performance",
            Profile::Custom => "custom",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Profile {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Profile::ALL
            .into_iter()
            .find(|profile| profile.as_str() == value)
            .ok_or_else(|| ConfigError::UnknownProfile(value.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_profile_through_its_name() {
        for profile in Profile::ALL {
            assert_eq!(profile.as_str().parse::<Profile>().unwrap(), profile);
        }
    }

    #[test]
    fn rejects_unknown_names() {
        let error = "turbo".parse::<Profile>().unwrap_err();
        assert!(matches!(error, ConfigError::UnknownProfile(name) if name == "turbo"));
    }

    #[test]
    fn displays_the_kernel_name() {
        assert_eq!(
            Profile::BalancedPerformance.to_string(),
            "balanced-performance"
        );
    }

    #[test]
    fn parsing_is_case_sensitive() {
        assert!("Quiet".parse::<Profile>().is_err());
    }
}
