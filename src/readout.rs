//! The readout: what the file says of the mesh the pointer is on, said
//! beside the pointer.

use eframe::egui::{Label, Response, Ui, Widget, vec2};

use crate::asset;
use crate::dataset::Mesh;

/// What the screen puts before the mesh's code, before the name of its
/// municipality, and before its value.
pub(crate) const CODE_LABEL: &str = asset::text!("mesh-code-label.txt");
pub(crate) const MUNICIPALITY_LABEL: &str = asset::text!("municipality-label.txt");
pub(crate) const VALUE_LABEL: &str = asset::text!("value-label.txt");

/// How wide the labels' column is, in points: the widest label, and a space
/// after it.
const LABELS_WIDTH: f32 = 100.0;

/// A mesh's code, the name of its municipality, and the value the colours
/// show for it, a line each under one another. Each is said in full: a name
/// or a value too long for the room it is given goes on in the line below.
pub(crate) struct Readout<'a> {
    mesh: &'a Mesh,
    /// The value as the screen writes it.
    value: String,
}

impl<'a> Readout<'a> {
    /// What is read out of `mesh`, for which the file publishes `value`. The
    /// value is written with the fewest figures that tell it from every
    /// other, so one the file writes that way reads as the file has it.
    pub(crate) fn published(mesh: &'a Mesh, value: f64) -> Self {
        let value = value.to_string();
        Self { mesh, value }
    }

    /// What is read out of `mesh`, whose weighted sum is `sum`. The sum is
    /// written as the files write their values: to ten significant figures,
    /// with no zeros after the last of them.
    pub(crate) fn summed(mesh: &'a Mesh, sum: f64) -> Self {
        let value = to_ten_figures(sum).to_string();
        Self { mesh, value }
    }

    /// Each line's label and what it says.
    fn lines(&self) -> [(&'static str, String); 3] {
        [
            (CODE_LABEL, self.mesh.code().to_owned()),
            (MUNICIPALITY_LABEL, self.mesh.city().to_owned()),
            (VALUE_LABEL, self.value.clone()),
        ]
    }
}

/// `value` rounded to ten significant figures: written out to ten and read
/// back, which rounds on its decimal digits. A zero has no sign.
fn to_ten_figures(value: f64) -> f64 {
    let written = format!("{value:.9e}");
    let rounded: f64 = written.parse().expect("a number written out reads back");
    // Adding zero turns a negative zero into zero and changes nothing else.
    rounded + 0.0
}

impl Widget for Readout<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for (label, said) in self.lines() {
                ui.horizontal_top(|ui| {
                    ui.allocate_ui(vec2(LABELS_WIDTH, 0.0), |ui| {
                        ui.set_min_width(LABELS_WIDTH);
                        ui.label(label);
                    });
                    // In the room the labels leave, and no wider.
                    ui.add(Label::new(said).wrap());
                });
            }
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{city_of, dataset_of_totals};

    #[test]
    fn a_mesh_is_read_out_as_its_code_its_municipality_and_its_value() {
        let dataset = dataset_of_totals(&[("543823431", -0.125)]);
        let readout = Readout::published(&dataset.meshes()[0], -0.125);
        assert_eq!(
            readout.lines(),
            [
                (CODE_LABEL, "543823431".to_owned()),
                (MUNICIPALITY_LABEL, city_of("543823431")),
                (VALUE_LABEL, "-0.125".to_owned()),
            ]
        );
    }

    /// A value is written in full however small or large, and a whole one
    /// without a point.
    #[test]
    fn a_value_is_written_with_every_figure_it_has() {
        let dataset = dataset_of_totals(&[("543823431", 1.0)]);
        let mesh = &dataset.meshes()[0];
        let written = |value| Readout::published(mesh, value).lines()[2].1.clone();
        assert_eq!(written(0.000_012_345_678), "0.000012345678");
        assert_eq!(written(-1234.5678), "-1234.5678");
        assert_eq!(written(3.0), "3");
    }

    /// What adding up leaves past the tenth figure is not written, and a sum
    /// of fewer figures is written with those it has.
    #[test]
    fn a_weighted_sum_is_written_to_ten_significant_figures() {
        let dataset = dataset_of_totals(&[("543823431", 1.0)]);
        let mesh = &dataset.meshes()[0];
        let written = |sum| Readout::summed(mesh, sum).lines()[2].1.clone();
        assert_eq!(0.1 + 0.2, 0.300_000_000_000_000_04);
        assert_eq!(written(0.1 + 0.2), "0.3");
        assert_eq!(written(1.0 / 3.0), "0.3333333333");
        assert_eq!(written(-200.0 / 3.0), "-66.66666667");
        assert_eq!(written(0.000_123_456_789_123), "0.0001234567891");
        assert_eq!(written(123_456_789_123.0), "123456789100");
        assert_eq!(written(2.5), "2.5");
        assert_eq!(written(0.0), "0");
        assert_eq!(written(-0.0), "0");
        assert_eq!(written(-1e-300 * 1e-300), "0");
    }
}
