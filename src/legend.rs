//! The legend: which colour stands for which value.

use eframe::egui::{Color32, Layout, Rect, Response, Sense, Shape, Ui, Widget, pos2, vec2};
use eframe::emath::Align;
use eframe::epaint::Mesh as Triangles;

use crate::layer::Paint;

/// How high the bar of colours is, in points.
const BAR_HEIGHT: f32 = 14.0;

/// How many steps the bar of colours is drawn in, enough to look continuous.
const STEPS: u16 = 64;

/// What the bar is drawn on, as a square is drawn on the base map: the pale
/// map is white over most of the land.
const GROUND: Color32 = Color32::WHITE;

/// A bar of the colours the map paints values in, from the lowest to the
/// highest, and under it the values its ends and its middle stand for.
pub(crate) struct Legend {
    paint: Paint,
}

impl Legend {
    pub(crate) fn of(paint: Paint) -> Self {
        Self { paint }
    }

    /// What the lower end, the middle, and the upper end stand for, as the
    /// screen writes them.
    fn marks(&self) -> [String; 3] {
        let reach = two_figures(self.paint.reach());
        [format!("-{reach}"), "0".to_owned(), format!("+{reach}")]
    }

    /// The bar, filling `bar`: the ground, then on it a step of each colour.
    fn colours(&self, bar: Rect) -> Shape {
        let mut triangles = Triangles::default();
        triangles.add_colored_rect(bar, GROUND);
        let step = bar.width() / f32::from(STEPS);
        for at in 0..STEPS {
            // The value halfway across this step, from minus the reach at
            // the bar's left to plus the reach at its right.
            let share = (f64::from(at) + 0.5) / f64::from(STEPS) * 2.0 - 1.0;
            let left = bar.left() + step * f32::from(at);
            triangles.add_colored_rect(
                Rect::from_min_size(pos2(left, bar.top()), vec2(step, bar.height())),
                self.paint.of(self.paint.reach() * share),
            );
        }
        Shape::mesh(triangles)
    }
}

impl Widget for Legend {
    fn ui(self, ui: &mut Ui) -> Response {
        let [lowest, middle, highest] = self.marks();
        ui.vertical(|ui| {
            let size = vec2(ui.available_width(), BAR_HEIGHT);
            let (bar, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().add(self.colours(bar));
            ui.columns(3, |columns| {
                columns[0].label(lowest);
                columns[1].vertical_centered(|ui| ui.label(middle));
                columns[2].with_layout(Layout::right_to_left(Align::Min), |ui| ui.label(highest));
            });
        })
        .response
    }
}

/// `value`, which is not negative, written to its first two figures and
/// zeros from there: a scale's reach has no more figures than that.
fn two_figures(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    // In scientific notation with one decimal the value is `d.de<exponent>`,
    // its two figures and where the first of them lies.
    let rounded = format!("{value:.1e}");
    let exponent: i32 = rounded
        .split_once('e')
        .and_then(|(_, exponent)| exponent.parse().ok())
        .expect("a number in scientific notation has an exponent");
    let rounded: f64 = rounded
        .parse()
        .expect("a number in scientific notation is a number");
    // One figure before the point leaves one after it, and each place the
    // first figure lies further right takes one more; further left, none.
    let decimals = usize::try_from(1 - exponent).unwrap_or(0);
    format!("{rounded:.decimals$}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::MeshLayer;
    use crate::test_support::dataset_of_totals;

    /// The paint of a map that shows one mesh, whose value is the reach.
    fn paint(reach: f64) -> Paint {
        let dataset = dataset_of_totals(&[("543823431", reach)]);
        MeshLayer::showing(dataset.meshes(), &dataset.series()[0]).paint()
    }

    fn marks(reach: f64) -> [String; 3] {
        Legend::of(paint(reach)).marks()
    }

    #[test]
    fn the_ends_are_marked_with_plus_and_minus_the_reach_and_the_middle_with_zero() {
        assert_eq!(marks(1.2), ["-1.2", "0", "+1.2"]);
    }

    #[test]
    fn a_reach_of_any_size_is_written_to_two_figures() {
        assert_eq!(marks(0.046)[2], "+0.046");
        assert_eq!(marks(2.0)[2], "+2.0");
        assert_eq!(marks(38.0)[2], "+38");
        assert_eq!(marks(1200.0)[2], "+1200");
        assert_eq!(marks(990000.0)[2], "+990000");
        assert_eq!(marks(3e-10)[2], "+0.00000000030");
    }

    /// The scale rounds its reach up to two figures, so the mark is the very
    /// value the colours were worked out from.
    #[test]
    fn the_marks_are_the_reach_the_scale_has() {
        for size in [1.234, 9.96, 0.0996, 987_654.0] {
            let paint = paint(size);
            let written: f64 = Legend::of(paint).marks()[2].parse().unwrap();
            assert_eq!(written, paint.reach(), "for {size}");
        }
    }

    #[test]
    fn a_scale_with_no_reach_is_marked_with_zeros() {
        assert_eq!(marks(0.0), ["-0", "0", "+0"]);
    }

    /// The bar is the ground with a step of each colour on it, from what the
    /// map paints a value just above minus the reach in, to what it paints
    /// one just below the reach in, and it fills the room it is given.
    #[test]
    fn the_bar_shows_the_colours_as_the_map_paints_them() {
        let paint = paint(4.0);
        let bar = Rect::from_min_size(pos2(10.0, 20.0), vec2(128.0, BAR_HEIGHT));
        let Shape::Mesh(triangles) = Legend::of(paint).colours(bar) else {
            panic!("the bar is not drawn as triangles");
        };
        let colours: Vec<Color32> = triangles
            .vertices
            .chunks(4)
            .map(|step| step[0].color)
            .collect();
        let [ground, steps @ ..] = colours.as_slice() else {
            panic!("the bar is empty");
        };
        let [first, .., last] = steps else {
            panic!("the bar has no steps");
        };
        let half_a_step = 4.0 / f64::from(STEPS);
        assert_eq!(*ground, GROUND);
        assert_eq!(steps.len(), usize::from(STEPS));
        assert_eq!(*first, paint.of(-4.0 + half_a_step));
        assert_eq!(*last, paint.of(4.0 - half_a_step));
        assert_eq!(steps[steps.len() / 2], paint.of(half_a_step));
        assert_ne!(first, last);
        assert_eq!(triangles.calc_bounds(), bar);
    }
}
