use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Temperature(i32);

impl Temperature {
    pub const fn from_millicelsius(value: i32) -> Self {
        Self(value)
    }

    pub const fn from_celsius(value: i32) -> Self {
        Self(value.saturating_mul(1000))
    }

    pub const fn millicelsius(self) -> i32 {
        self.0
    }

    pub fn celsius(self) -> f64 {
        f64::from(self.0) / 1000.0
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.1} °C", self.celsius())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_celsius_to_millicelsius() {
        assert_eq!(Temperature::from_celsius(42).millicelsius(), 42_000);
    }

    #[test]
    fn saturates_instead_of_overflowing() {
        assert_eq!(Temperature::from_celsius(i32::MAX).millicelsius(), i32::MAX);
    }

    #[test]
    fn orders_by_value() {
        assert!(Temperature::from_celsius(50) > Temperature::from_millicelsius(49_999));
    }

    #[test]
    fn formats_with_one_decimal() {
        assert_eq!(
            Temperature::from_millicelsius(42_050).to_string(),
            "42.0 °C"
        );
    }

    #[test]
    fn exposes_fractional_celsius() {
        assert!((Temperature::from_millicelsius(39_050).celsius() - 39.05).abs() < f64::EPSILON);
    }
}
