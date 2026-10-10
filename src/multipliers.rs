//! The multipliers: how many times its weight in the published data the
//! user gives each indicator, and the weighted sums they make of a file's
//! values.

use crate::dataset::Dataset;

/// A multiplier for each indicator of a file, in the order the file's
/// dataset has them.
#[derive(Debug, PartialEq)]
pub(crate) struct Multipliers {
    each: Vec<f64>,
}

impl Multipliers {
    /// Every indicator of `dataset` at 1, its weight in the published data.
    pub(crate) fn ones_for(dataset: &Dataset) -> Self {
        Self {
            each: vec![1.0; dataset.indicators().len()],
        }
    }

    /// The weighted sum of each mesh of `dataset`, in the order of its
    /// meshes: the mesh's value of each indicator times that indicator's
    /// multiplier, added up in the order of the indicators.
    pub(crate) fn weighted_sums(&self, dataset: &Dataset) -> Vec<f64> {
        let mut sums = vec![0.0; dataset.meshes().len()];
        for (indicator, multiplier) in dataset.indicators().iter().zip(&self.each) {
            for (sum, value) in sums.iter_mut().zip(indicator.values()) {
                *sum += value * multiplier;
            }
        }
        sums
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::dataset::Mesh;
    use crate::test_support::{dataset_of, real_files};

    const MESHES: [&str; 2] = ["543823431", "543823432"];

    /// The total is no indicator: a file whose total is not the sum of its
    /// indicators has the sum all the same.
    #[test]
    fn with_every_multiplier_at_1_a_weighted_sum_is_the_sum_of_the_indicators() {
        let dataset = dataset_of(
            &MESHES,
            &[
                ("A01", "Stations", &[0.25, -1.5]),
                ("QOL", "Total", &[100.0, 100.0]),
                ("B02", "Floods", &[0.5, 4.0]),
            ],
        );
        let sums = Multipliers::ones_for(&dataset).weighted_sums(&dataset);
        assert_eq!(sums, [0.75, 2.5]);
    }

    #[test]
    fn each_indicator_counts_as_many_times_as_its_multiplier() {
        let dataset = dataset_of(
            &MESHES,
            &[
                ("A01", "Stations", &[0.25, -1.5]),
                ("B02", "Floods", &[0.5, 4.0]),
                ("QOL", "Total", &[0.75, 2.5]),
            ],
        );
        let multipliers = Multipliers {
            each: vec![2.0, 0.5],
        };
        assert_eq!(multipliers.weighted_sums(&dataset), [0.75, -1.0]);
    }

    #[test]
    fn a_file_with_no_indicator_has_a_weighted_sum_of_zero_for_every_mesh() {
        let dataset = dataset_of(&MESHES, &[("QOL", "Total", &[0.75, 2.5])]);
        let sums = Multipliers::ones_for(&dataset).weighted_sums(&dataset);
        assert_eq!(sums, [0.0, 0.0]);
    }

    /// What a second reading of a real file, which shares no code with the
    /// tool's own, finds for one mesh.
    #[derive(Default)]
    struct Read {
        /// Its indicator rows added up in the file's order.
        sum: f64,
        /// Their sizes added up.
        size: f64,
        /// How far the sum of the rows as written may lie from the total as
        /// written, where both come of the same figures: one in the tenth
        /// significant figure of each row, the total's with them.
        rounding: f64,
        indicators: u32,
        total: Option<f64>,
    }

    /// One in the tenth significant figure of `value`, past which the files
    /// write nothing, whether they round to it or cut there. Zero has no
    /// figure to be out by.
    fn one_in_the_tenth_figure(value: f64) -> f64 {
        if value == 0.0 {
            return 0.0;
        }
        // The exponent of a finite number that is not zero is a small
        // whole number.
        #[expect(clippy::cast_possible_truncation)]
        let first = value.abs().log10().floor() as i32;
        10f64.powi(first - 9)
    }

    /// How many significant figures `written` has: its digits from the first
    /// that is not a zero.
    fn figures(written: &str) -> usize {
        let digits: String = written.chars().filter(char::is_ascii_digit).collect();
        digits.trim_start_matches('0').len()
    }

    /// Every real file: with every multiplier at 1 the tool's weighted sums
    /// are the sums of the file's indicator rows. A file's totals are those
    /// sums in every mesh or in none, and at least one file's are. No value
    /// is written with more than ten significant figures.
    #[test]
    #[ignore = "reads the real files in the folder QOL_REWEIGHT_REAL_FILES names"]
    fn a_real_files_weighted_sums_at_1_are_the_sums_of_its_indicator_rows() {
        let mut whose_total_is_the_sum = 0;
        for path in real_files() {
            let name = path.display();
            let text = std::fs::read_to_string(&path).expect("the file is UTF-8");
            let mut rows = csv::Reader::from_reader(text.trim_start_matches('\u{feff}').as_bytes());
            let column = |name: &str| {
                let headers = rows.headers().expect("a header row");
                headers
                    .iter()
                    .position(|header| header == name)
                    .expect("the column")
            };
            let [key_code, indicator_code, value] =
                ["KeyCode", "IndicatorCode", "Value"].map(column);
            let mut read: HashMap<String, Read> = HashMap::new();
            for row in rows.records() {
                let row = row.expect("a row");
                let written = &row[value];
                assert!(
                    figures(written) <= 10,
                    "{name}: mesh {} has a value of more than ten figures",
                    &row[key_code]
                );
                let number: f64 = written.parse().expect("a number");
                let mesh = read.entry(row[key_code].to_owned()).or_default();
                mesh.rounding += one_in_the_tenth_figure(number);
                if &row[indicator_code] == "QOL" {
                    mesh.total = Some(number);
                } else {
                    mesh.sum += number;
                    mesh.size += number.abs();
                    mesh.indicators += 1;
                }
            }

            let dataset = Dataset::open(path.as_os_str()).expect("the tool reads the file");
            let sums = Multipliers::ones_for(&dataset).weighted_sums(&dataset);
            let codes = dataset.meshes().iter().map(Mesh::code);
            let mut total_is_the_sum = 0;
            for (code, sum) in codes.zip(&sums) {
                let read = &read[code];
                // Adding in another order differs by no more than this.
                let adding = f64::from(read.indicators) * f64::EPSILON * read.size;
                assert!(
                    (sum - read.sum).abs() <= adding,
                    "{name}: the weighted sum of mesh {code} is not the sum of its rows"
                );
                let total = read
                    .total
                    .expect("the tool read the file, so it has a total");
                if (sum - total).abs() <= read.rounding + adding {
                    total_is_the_sum += 1;
                }
            }
            if total_is_the_sum == sums.len() {
                whose_total_is_the_sum += 1;
                println!("{name}: the total is the sum of the indicators in every mesh");
            } else {
                assert!(
                    total_is_the_sum == 0,
                    "{name}: the total is the sum of the indicators in some meshes alone"
                );
                println!("{name}: the total is the sum of the indicators in no mesh");
            }
        }
        assert!(
            whose_total_is_the_sum > 0,
            "no file's total is the sum of its indicators"
        );
    }
}
