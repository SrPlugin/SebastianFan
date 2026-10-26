use std::path::Path;

use sebastianfan_core::Config;

use crate::error::CliError;
use crate::render::table;
use crate::store;

pub fn show(config: &Config) -> String {
    let rows: Vec<Vec<String>> = config
        .curve
        .steps()
        .iter()
        .map(|step| {
            vec![
                format!(">= {} °C", step.min_celsius),
                step.profile.to_string(),
            ]
        })
        .collect();
    table(&rows)
}

pub fn validate(file: &Path) -> Result<String, CliError> {
    let config = store::read(file)?;
    Ok(format!(
        "{}: ok ({} steps)",
        file.display(),
        config.curve.steps().len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_default_curve_aligned() {
        assert_eq!(
            show(&Config::default()),
            ">= 0 °C   quiet\n>= 55 °C  balanced\n>= 75 °C  balanced-performance"
        );
    }
}
