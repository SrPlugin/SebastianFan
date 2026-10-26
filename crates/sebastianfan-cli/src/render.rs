use std::fmt::Display;

use sebastianfan_core::HardwareError;

pub fn table(rows: &[Vec<String>]) -> String {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    let widths: Vec<usize> = (0..columns)
        .map(|column| {
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| cell.chars().count())
                .max()
                .unwrap_or(0)
        })
        .collect();
    rows.iter()
        .map(|row| {
            let last = row.len().saturating_sub(1);
            row.iter()
                .enumerate()
                .map(|(index, cell)| match widths.get(index) {
                    Some(width) if index != last => format!("{cell:<width$}"),
                    _ => cell.clone(),
                })
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn describe<T: Display>(result: Result<T, HardwareError>) -> String {
    match result {
        Ok(value) => value.to_string(),
        Err(error) => format!("unavailable ({error})"),
    }
}

pub fn print_block(text: &str) {
    if !text.is_empty() {
        println!("{text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|cell| (*cell).to_owned()).collect()
    }

    #[test]
    fn aligns_columns_to_the_widest_cell() {
        let rows = [row(&["a", "first", "1"]), row(&["longer", "x", "22"])];
        assert_eq!(table(&rows), "a       first  1\nlonger  x      22");
    }

    #[test]
    fn leaves_the_last_column_unpadded() {
        let rows = [row(&["a", "short"]), row(&["b", "much longer"])];
        assert!(table(&rows).lines().all(|line| !line.ends_with(' ')));
    }

    #[test]
    fn counts_characters_not_bytes() {
        let rows = [row(&["42.0 °C", "x"]), row(&["1.0 °C", "y"])];
        assert_eq!(table(&rows), "42.0 °C  x\n1.0 °C   y");
    }

    #[test]
    fn renders_nothing_for_no_rows() {
        assert_eq!(table(&[]), "");
    }

    #[test]
    fn describes_failures_instead_of_propagating_them() {
        let result: Result<u32, HardwareError> = Err(HardwareError::NotFound("fan".into()));
        assert_eq!(describe(result), "unavailable (fan not found)");
    }

    #[test]
    fn describes_successful_values() {
        assert_eq!(describe::<u32>(Ok(7)), "7");
    }
}
