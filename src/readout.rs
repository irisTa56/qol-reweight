//! The readout: what the file says of the mesh the pointer is on.

use eframe::egui::{Label, Response, Ui, Widget, vec2};

use crate::asset;
use crate::dataset::Mesh;

/// What the screen puts before the mesh's code, before the name of its
/// municipality, and before its value.
pub(crate) const CODE_LABEL: &str = asset::text!("mesh-code-label.txt");
pub(crate) const MUNICIPALITY_LABEL: &str = asset::text!("municipality-label.txt");
pub(crate) const VALUE_LABEL: &str = asset::text!("value-label.txt");

/// What stands for each of the three while the pointer is on no mesh, so
/// that the readout takes the same room with a mesh and without.
const NOTHING: &str = "-";

/// How wide the labels' column is, in points: the widest label, and a space
/// after it.
const LABELS_WIDTH: f32 = 100.0;

/// A mesh's code, the name of its municipality, and the value the colours
/// show for it, a line each under one another. It is as wide as the room it
/// is given whatever it says: a name or a value too long for its line goes
/// on in the line below.
pub(crate) struct Readout<'a> {
    /// The mesh the pointer is on and its value, or none where it is on no
    /// mesh.
    pointed: Option<(&'a Mesh, f64)>,
}

impl<'a> Readout<'a> {
    pub(crate) fn of(pointed: Option<(&'a Mesh, f64)>) -> Self {
        Self { pointed }
    }

    /// Each line's label and what it says. The value is written with the
    /// fewest figures that tell it from every other, so one the file writes
    /// that way reads as the file has it.
    fn lines(&self) -> [(&'static str, String); 3] {
        let [code, city, value] = match self.pointed {
            Some((mesh, value)) => [
                mesh.code().to_owned(),
                mesh.city().to_owned(),
                value.to_string(),
            ],
            None => [NOTHING; 3].map(str::to_owned),
        };
        [
            (CODE_LABEL, code),
            (MUNICIPALITY_LABEL, city),
            (VALUE_LABEL, value),
        ]
    }
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
        let readout = Readout::of(Some((&dataset.meshes()[0], -0.125)));
        assert_eq!(
            readout.lines(),
            [
                (CODE_LABEL, "543823431".to_owned()),
                (MUNICIPALITY_LABEL, city_of("543823431")),
                (VALUE_LABEL, "-0.125".to_owned()),
            ]
        );
    }

    #[test]
    fn no_mesh_is_read_out_as_the_same_lines_with_nothing_in_them() {
        let said = Readout::of(None).lines().map(|(_, said)| said);
        assert_eq!(said, [NOTHING; 3]);
    }

    /// A value is written in full however small or large, and a whole one
    /// without a point.
    #[test]
    fn a_value_is_written_with_every_figure_it_has() {
        let dataset = dataset_of_totals(&[("543823431", 1.0)]);
        let mesh = &dataset.meshes()[0];
        let written = |value| Readout::of(Some((mesh, value))).lines()[2].1.clone();
        assert_eq!(written(0.000_012_345_678), "0.000012345678");
        assert_eq!(written(-1234.5678), "-1234.5678");
        assert_eq!(written(3.0), "3");
    }
}
