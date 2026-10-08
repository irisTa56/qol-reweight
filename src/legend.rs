//! The legend: which colour stands for which value.

use eframe::egui::{Color32, Layout, Rect, Response, Sense, Shape, Ui, Widget, pos2, vec2};
use eframe::emath::Align;
use eframe::epaint::Mesh as Triangles;
use unit_prefix::NumberPrefix;

use crate::paint::Paint;

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
    /// screen writes them. The reach has two figures at most, which is the
    /// scale's doing; from a thousand up it is written with a prefix, k or M
    /// and so on, in place of its zeros.
    fn marks(&self) -> [String; 3] {
        let reach = match NumberPrefix::decimal(self.paint.reach()) {
            NumberPrefix::Standalone(reach) => format!("{reach}"),
            NumberPrefix::Prefixed(prefix, reach) => format!("{reach}{prefix}"),
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The paint of a map whose values reach `reach`.
    fn paint(reach: f64) -> Paint {
        Paint::fitting(&[reach])
    }

    fn marks(reach: f64) -> [String; 3] {
        Legend::of(paint(reach)).marks()
    }

    #[test]
    fn the_ends_are_marked_with_plus_and_minus_the_reach_and_the_middle_with_zero() {
        assert_eq!(marks(1.2), ["-1.2", "0", "+1.2"]);
    }

    /// The scale rounds its reach to two figures, and the mark is that
    /// reach, the very value the colours were worked out from, with no
    /// figure added and none dropped.
    #[test]
    fn the_marks_are_the_reach_the_scale_has() {
        assert_eq!(marks(1.234)[2], "+1.2");
        assert_eq!(marks(2.0)[2], "+2");
        assert_eq!(marks(0.0456)[2], "+0.046");
        assert_eq!(marks(9.96)[2], "+10");
        assert_eq!(marks(990.0)[2], "+990");
        for size in [1.234, 9.96, 0.0996, 987.0] {
            let paint = paint(size);
            let written: f64 = Legend::of(paint).marks()[2].parse().unwrap();
            assert_eq!(written, paint.reach(), "for {size}");
        }
    }

    /// k for thousands, M for millions, G for thousands of millions.
    #[test]
    fn a_reach_of_a_thousand_or_more_is_written_with_a_prefix() {
        assert_eq!(marks(1000.0), ["-1k", "0", "+1k"]);
        assert_eq!(marks(1234.0)[2], "+1.2k");
        assert_eq!(marks(987_654.0)[2], "+990k");
        assert_eq!(marks(4_321_000.0)[2], "+4.3M");
        assert_eq!(marks(77_000_000.0)[2], "+77M");
        assert_eq!(marks(2e9)[2], "+2G");
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
