//! The readout: what the file says of the mesh the pointer is on.

use eframe::egui::{Grid, Response, Ui, Widget};

use crate::dataset::Mesh;

/// What the screen puts before the mesh's code, before the name of its
/// municipality, and before its value.
pub(crate) const CODE_LABEL: &str = include_str!("../assets/mesh-code-label.txt").trim_ascii_end();
pub(crate) const MUNICIPALITY_LABEL: &str =
    include_str!("../assets/municipality-label.txt").trim_ascii_end();
pub(crate) const VALUE_LABEL: &str = include_str!("../assets/value-label.txt").trim_ascii_end();

/// What stands for each of the three while the pointer is on no mesh, so
/// that the readout takes the same room with a mesh and without.
const NOTHING: &str = "-";

/// A mesh's code, the name of its municipality, and the value the colours
/// show for it, a line each under one another.
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
        Grid::new("readout")
            .num_columns(2)
            .show(ui, |ui| {
                for (label, said) in self.lines() {
                    ui.label(label);
                    ui.label(said);
                    ui.end_row();
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
