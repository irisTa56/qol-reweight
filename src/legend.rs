//! The legend: which colour stands for which value.

use eframe::egui::{Color32, Layout, Rect, Response, Sense, Shape, Ui, Widget, pos2, vec2};
use eframe::emath::Align;
use eframe::epaint::Mesh as Triangles;

use crate::scale::Scale;

/// How high the bar of colours is, in points.
const BAR_HEIGHT: f32 = 14.0;

/// How many steps the bar of colours is drawn in, enough to look continuous.
const STEPS: u16 = 64;

/// A bar of a scale's colours from its lower end to its upper one, and under
/// it the values its ends and its middle stand for.
pub(crate) struct Legend {
    scale: Scale,
}

impl Legend {
    pub(crate) fn of(scale: Scale) -> Self {
        Self { scale }
    }

    /// What the lower end, the middle, and the upper end stand for, as the
    /// screen writes them.
    fn marks(&self) -> [String; 3] {
        let reach = two_figures(self.scale.reach());
        [format!("-{reach}"), "0".to_owned(), format!("+{reach}")]
    }

    /// The bar, filling `bar`.
    fn colours(&self, bar: Rect) -> Shape {
        let mut triangles = Triangles::default();
        let step = bar.width() / f32::from(STEPS);
        for at in 0..STEPS {
            // The value halfway across this step, from minus the reach at
            // the bar's left to plus the reach at its right.
            let share = (f64::from(at) + 0.5) / f64::from(STEPS) * 2.0 - 1.0;
            let colour = self.scale.colour(self.scale.reach() * share);
            let left = bar.left() + step * f32::from(at);
            triangles.add_colored_rect(
                Rect::from_min_size(pos2(left, bar.top()), vec2(step, bar.height())),
                Color32::from_rgb(colour.r, colour.g, colour.b),
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

/// `value`, which is not negative, written to two figures: enough to tell
/// one scale from another, whatever size the values of a file have.
fn two_figures(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    // One figure before the point leaves one after it, and each place the
    // first figure lies further right takes one more.
    let decimals = (1.0 - value.log10().floor()).clamp(0.0, 9.0) as usize;
    format!("{value:.decimals$}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marks(reach: f64) -> [String; 3] {
        Legend::of(Scale::fitting(&[reach])).marks()
    }

    #[test]
    fn the_ends_are_marked_with_plus_and_minus_the_reach_and_the_middle_with_zero() {
        assert_eq!(marks(1.234), ["-1.2", "0", "+1.2"]);
    }

    #[test]
    fn a_reach_of_any_size_is_written_to_two_figures() {
        assert_eq!(marks(0.0456)[2], "+0.046");
        assert_eq!(marks(2.0)[2], "+2.0");
        assert_eq!(marks(37.5)[2], "+38");
        assert_eq!(marks(1234.0)[2], "+1234");
    }

    #[test]
    fn a_scale_with_no_reach_is_marked_with_zeros() {
        assert_eq!(marks(0.0), ["-0", "0", "+0"]);
    }

    /// The bar runs from the colour of the lower end, through the neutral
    /// one, to the colour of the upper end.
    #[test]
    fn the_bar_runs_through_the_colours_of_the_scale() {
        let scale = Scale::fitting(&[4.0]);
        let bar = Rect::from_min_size(pos2(10.0, 20.0), vec2(128.0, BAR_HEIGHT));
        let Shape::Mesh(triangles) = Legend::of(scale).colours(bar) else {
            panic!("the bar is not drawn as triangles");
        };
        let colours: Vec<Color32> = triangles
            .vertices
            .chunks(4)
            .map(|step| step[0].color)
            .collect();
        let [first, .., last] = colours.as_slice() else {
            panic!("the bar has no steps");
        };
        let middle = colours[colours.len() / 2];
        assert!(first.r() > first.b() + 50, "{first:?} is not red");
        assert!(last.b() > last.r() + 50, "{last:?} is not blue");
        assert!(
            middle.r().abs_diff(middle.b()) < 30,
            "{middle:?} is not neutral"
        );
        assert_eq!(triangles.calc_bounds(), bar);
    }
}
