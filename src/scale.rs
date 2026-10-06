//! How a published value becomes a colour.
//!
//! A published value is a difference from a mean, so its sign carries
//! meaning: the colours run from one hue through a neutral colour at zero to
//! another hue.

use colorous::{Color, Gradient};

/// The colours of the scale: red for values below zero, blue for values
/// above, and a pale grey between them.
const COLOURS: Gradient = colorous::RED_BLUE;

/// The share of the values shown whose size the two ends of the scale cover.
/// The few beyond it take the colour of an end, which keeps one extreme mesh
/// from leaving all the others near the neutral colour.
const COVERED: f64 = 0.98;

/// The scale for one set of values: which value its ends stand for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Scale {
    /// The ends stand for plus and minus this. Never negative, and zero only
    /// where every value covered is zero.
    reach: f64,
}

impl Scale {
    /// The scale that fits `values`: its ends stand for the size that
    /// [`COVERED`] of them do not exceed.
    pub(crate) fn fitting(values: &[f64]) -> Self {
        let mut sizes: Vec<f64> = values.iter().map(|value| value.abs()).collect();
        sizes.sort_by(f64::total_cmp);
        // The smallest size that at least `COVERED` of the sizes do not
        // exceed.
        let covered = (sizes.len() as f64 * COVERED).ceil() as usize;
        let reach = covered
            .checked_sub(1)
            .and_then(|at| sizes.get(at))
            .copied()
            .unwrap_or(0.0);
        Self { reach }
    }

    /// The value the upper end stands for; the lower end stands for its
    /// negative.
    pub(crate) fn reach(self) -> f64 {
        self.reach
    }

    /// The colour of `value`.
    pub(crate) fn colour(self, value: f64) -> Color {
        COLOURS.eval_continuous(self.place(value))
    }

    /// Where `value` is on the scale: 0 at the lower end, 0.5 at zero, and 1
    /// at the upper end, in step with the value between them.
    fn place(self, value: f64) -> f64 {
        let size = value.abs();
        // How far toward an end: all the way from the reach on. With no
        // reach, any value but zero is beyond it.
        let share = if size >= self.reach && size > 0.0 {
            1.0
        } else if size > 0.0 {
            size / self.reach
        } else {
            0.0
        };
        0.5 + share.copysign(value) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use proptest::collection::vec;
    use proptest::prelude::*;

    use super::*;

    /// Any value a file can hold.
    fn value() -> impl Strategy<Value = f64> {
        prop::num::f64::NORMAL | prop::num::f64::ZERO
    }

    /// A scale with some reach, small enough that a test can go ten times
    /// beyond it and stay within the numbers there are.
    fn scale() -> impl Strategy<Value = Scale> {
        (1e-300..1e300f64).prop_map(|reach| Scale::fitting(&[reach]))
    }

    /// The colour at a place along the colours, as its red, green, and blue.
    fn at(place: f64) -> [u8; 3] {
        COLOURS.eval_continuous(place).as_array()
    }

    /// The colour the scale gives a value, likewise.
    fn colour(scale: Scale, value: f64) -> [u8; 3] {
        scale.colour(value).as_array()
    }

    #[test]
    fn a_few_extreme_values_do_not_stretch_the_scale() {
        let mut values = vec![1.0; 99];
        values.push(-1000.0);
        assert_eq!(Scale::fitting(&values).reach(), 1.0);
    }

    /// Of a hundred sizes, the scale reaches the ninety-eighth.
    #[test]
    fn the_scale_covers_98_in_100_of_the_values() {
        let values: Vec<f64> = (1..=100).map(|size| -f64::from(size)).collect();
        assert_eq!(Scale::fitting(&values).reach(), 98.0);
    }

    #[test]
    fn no_values_give_a_scale_with_no_reach() {
        assert_eq!(Scale::fitting(&[]).reach(), 0.0);
    }

    /// Red below zero and blue above it, deeper toward each end, with a
    /// colour that is neither at zero.
    #[test]
    fn the_colours_run_from_red_through_a_neutral_colour_to_blue() {
        let scale = Scale::fitting(&[4.0]);
        let [lowest, low, zero, high, highest] =
            [-4.0, -2.0, 0.0, 2.0, 4.0].map(|v| colour(scale, v));
        let redness = |[r, _, b]: [u8; 3]| i32::from(r) - i32::from(b);
        let lightness = |[r, g, b]: [u8; 3]| u32::from(r) + u32::from(g) + u32::from(b);
        assert!(
            redness(lowest) > 50 && redness(low) > 50,
            "{lowest:?} {low:?}"
        );
        assert!(
            redness(highest) < -50 && redness(high) < -50,
            "{high:?} {highest:?}"
        );
        assert!(redness(zero).abs() < 10, "{zero:?}");
        assert!(lightness(lowest) < lightness(low) && lightness(low) < lightness(zero));
        assert!(lightness(highest) < lightness(high) && lightness(high) < lightness(zero));
    }

    proptest! {
        /// The reach is the size of one of the values, the smallest that
        /// enough of them do not exceed.
        #[test]
        fn the_reach_is_the_smallest_size_that_covers_enough(values in vec(value(), 1..200)) {
            let reach = Scale::fitting(&values).reach();
            let enough = (values.len() as f64 * COVERED).ceil() as usize;
            let within = values.iter().filter(|v| v.abs() <= reach).count();
            let below = values.iter().filter(|v| v.abs() < reach).count();
            prop_assert!(values.iter().any(|v| v.abs() == reach));
            prop_assert!(within >= enough);
            prop_assert!(below < enough);
        }

        #[test]
        fn zero_is_neutral(values in vec(value(), 0..50)) {
            let scale = Scale::fitting(&values);
            prop_assert_eq!(colour(scale, 0.0), at(0.5));
            prop_assert_eq!(colour(scale, -0.0), at(0.5));
        }

        /// From the reach on, a value has the colour of its end.
        #[test]
        fn the_ends_are_at_the_reach_and_beyond(scale in scale(), beyond in 1.0..10.0f64) {
            let value = scale.reach() * beyond;
            prop_assert_eq!(colour(scale, scale.reach()), at(1.0));
            prop_assert_eq!(colour(scale, -scale.reach()), at(0.0));
            prop_assert_eq!(colour(scale, value), at(1.0));
            prop_assert_eq!(colour(scale, -value), at(0.0));
        }

        /// Where the values covered are all zero, any other value is beyond.
        #[test]
        fn without_a_reach_any_other_value_is_at_an_end(value in value()) {
            prop_assume!(value != 0.0);
            let scale = Scale::fitting(&[0.0]);
            let end = if value < 0.0 { at(0.0) } else { at(1.0) };
            prop_assert_eq!(colour(scale, value), end);
        }

        /// A value part of the way to the reach is that part of the way from
        /// the middle of the colours to an end, and its negative as far the
        /// other way.
        #[test]
        fn a_value_between_is_as_far_along_as_it_is_toward_the_reach(
            scale in scale(),
            share in 0.0..1.0f64,
        ) {
            let value = scale.reach() * share;
            let share = value / scale.reach();
            prop_assert_eq!(colour(scale, value), at(0.5 + share / 2.0));
            prop_assert_eq!(colour(scale, -value), at(0.5 - share / 2.0));
        }
    }
}
