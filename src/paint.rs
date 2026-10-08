//! How a value is painted on the map, and so in the legend of the map.

use eframe::egui::Color32;

use crate::scale::Scale;

/// How much of a square's colour covers the base map under it when the tool
/// starts, of 255. The rest shows through, which keeps the place names
/// readable.
pub(crate) const OPENING_OPACITY: u8 = 170;

/// How a value is painted on the map: in its colour on a scale, covering
/// the base map as much as its user sets.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Paint {
    scale: Scale,
    /// How much of a colour covers the base map, of 255.
    opacity: u8,
}

impl Paint {
    /// The paint for `values`, on the scale that fits them, covering
    /// `opacity` of 255 of the base map.
    pub(crate) fn fitting(values: &[f64], opacity: u8) -> Self {
        Self {
            scale: Scale::fitting(values),
            opacity,
        }
    }

    /// The colour a square with `value` is filled with.
    pub(crate) fn of(self, value: f64) -> Color32 {
        let colour = self.scale.colour(value);
        Color32::from_rgba_unmultiplied(colour.r, colour.g, colour.b, self.opacity)
    }

    /// The value the deepest colour one way stands for, and its negative the
    /// other way.
    pub(crate) fn reach(self) -> f64 {
        self.scale.reach()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_has_its_colour_on_the_scale_and_lets_the_base_map_through() {
        let values = [-2.0, 0.0, 1.0];
        let paint = Paint::fitting(&values, OPENING_OPACITY);
        let scale = Scale::fitting(&values);
        for value in values {
            let on_the_scale = scale.colour(value);
            let expected = Color32::from_rgba_unmultiplied(
                on_the_scale.r,
                on_the_scale.g,
                on_the_scale.b,
                OPENING_OPACITY,
            );
            assert_eq!(paint.of(value), expected, "for {value}");
            assert!(paint.of(value).a() < 255, "{value} hides the base map");
        }
        assert_ne!(paint.of(-2.0), paint.of(0.0));
        assert_ne!(paint.of(0.0), paint.of(1.0));
    }

    /// From a colour that leaves the base map as it is to one that hides it.
    #[test]
    fn a_colour_covers_as_much_of_the_base_map_as_is_set() {
        let values = [-2.0, 0.0, 1.0];
        for opacity in [0, 1, 99, 255] {
            let paint = Paint::fitting(&values, opacity);
            for value in values {
                assert_eq!(paint.of(value).a(), opacity, "for {value}");
            }
        }
    }

    #[test]
    fn the_reach_is_the_scales() {
        assert_eq!(
            Paint::fitting(&[0.5, -3.0], OPENING_OPACITY).reach(),
            Scale::fitting(&[0.5, -3.0]).reach()
        );
    }
}
