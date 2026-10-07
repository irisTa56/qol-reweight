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
    /// [`COVERED`] of them do not exceed, rounded to two figures, so that a
    /// legend can write the very value the colours were worked out from.
    pub(crate) fn fitting(values: &[f64]) -> Self {
        let mut sizes: Vec<f64> = values.iter().map(|value| value.abs()).collect();
        sizes.sort_by(f64::total_cmp);
        // The smallest size that at least `COVERED` of the sizes do not
        // exceed.
        let covered = (sizes.len() as f64 * COVERED).ceil() as usize;
        let smallest = covered
            .checked_sub(1)
            .and_then(|at| sizes.get(at))
            .copied()
            .unwrap_or(0.0);
        Self {
            reach: to_two_figures(smallest),
        }
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

/// `size` rounded to two figures: written out to two and read back, which
/// rounds on its decimal digits. The largest number there is stays itself,
/// where rounding would pass it.
fn to_two_figures(size: f64) -> f64 {
    let written = format!("{size:.1e}");
    let rounded: f64 = written.parse().expect("a number written out reads back");
    rounded.min(f64::MAX)
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

    /// A size of two figures or fewer is its own reach, and any other goes
    /// to the nearest one, into another power of ten where that is nearest.
    #[test]
    fn a_size_is_rounded_to_two_figures() {
        let reach = |size: f64| Scale::fitting(&[size]).reach();
        assert_eq!(reach(0.12), 0.12);
        assert_eq!(reach(0.3), 0.3);
        assert_eq!(reach(7.0), 7.0);
        assert_eq!(reach(123.0), 120.0);
        assert_eq!(reach(127.0), 130.0);
        assert_eq!(reach(0.1201), 0.12);
        assert_eq!(reach(99.6), 100.0);
        assert_eq!(reach(0.0996), 0.1);
        assert_eq!(reach(f64::MAX), f64::MAX);
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
        /// The reach has two figures and is within a twentieth of the
        /// smallest size that enough of the values do not exceed, which is
        /// as far as rounding to two figures can take it.
        #[test]
        fn the_reach_is_two_figures_near_the_size_that_covers_enough(
            values in vec(value(), 1..200),
        ) {
            let enough = (values.len() as f64 * COVERED).ceil() as usize;
            let mut sizes: Vec<f64> = values.iter().map(|v| v.abs()).collect();
            sizes.sort_by(f64::total_cmp);
            let smallest = sizes[enough - 1];

            let reach = Scale::fitting(&values).reach();
            let written = format!("{reach:e}");
            let figures = written.bytes().take_while(|b| *b != b'e').filter(u8::is_ascii_digit);
            prop_assert!((reach - smallest).abs() <= smallest * 0.05, "{reach} for {smallest}");
            // But for the largest number there is, which is kept as it is.
            prop_assert!(figures.count() <= 2 || reach == f64::MAX, "{written}");
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
