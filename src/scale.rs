//! How a published value becomes a colour.
//!
//! A published value is a difference from a mean, so its sign carries
//! meaning: the colours run from one hue through a neutral colour at zero to
//! another hue.

/// The share of the values shown whose size the two ends of the scale cover.
/// The few beyond it take the colour of an end, which keeps one extreme mesh
/// from leaving all the others near the neutral colour.
const COVERED: f64 = 0.98;

/// A colour, as its red, green, and blue parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rgb(pub(crate) [u8; 3]);

impl Rgb {
    /// The colour of zero.
    pub(crate) const NEUTRAL: Self = Self([247, 247, 247]);
    /// The colour of the lower end, a value as far below zero as the scale
    /// reaches.
    pub(crate) const BELOW: Self = Self([178, 24, 43]);
    /// The colour of the upper end.
    pub(crate) const ABOVE: Self = Self([33, 102, 172]);

    /// The colour `share` of the way from the neutral colour to `end`,
    /// `share` being from 0 to 1.
    fn toward(end: Self, share: f64) -> Self {
        let part = |i: usize| {
            let from = f64::from(Self::NEUTRAL.0[i]);
            let to = f64::from(end.0[i]);
            // Between two bytes, so it is one.
            (from + (to - from) * share).round() as u8
        };
        Self([part(0), part(1), part(2)])
    }
}

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
    pub(crate) fn colour(self, value: f64) -> Rgb {
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
        let end = if value < 0.0 { Rgb::BELOW } else { Rgb::ABOVE };
        Rgb::toward(end, share)
    }
}

#[cfg(test)]
mod tests {
    use proptest::collection::vec;
    use proptest::prelude::*;

    use super::*;

    /// Values of the size the published ones have, zero among them.
    fn value() -> impl Strategy<Value = f64> {
        prop_oneof![Just(0.0), -1e6..1e6]
    }

    fn scale() -> impl Strategy<Value = Scale> {
        vec(value(), 0..50).prop_map(|values| Scale::fitting(&values))
    }

    #[test]
    fn a_few_extreme_values_do_not_stretch_the_scale() {
        let mut values = vec![1.0; 99];
        values.push(-1000.0);
        assert_eq!(Scale::fitting(&values).reach(), 1.0);
    }

    #[test]
    fn no_values_give_a_scale_with_no_reach() {
        assert_eq!(Scale::fitting(&[]).reach(), 0.0);
    }

    proptest! {
        /// The reach is the size of one of the values, the smallest that
        /// enough of them do not exceed.
        #[test]
        fn the_reach_is_the_smallest_size_that_covers_enough(values in vec(value(), 1..200)) {
            let reach = Scale::fitting(&values).reach();
            let enough = (values.len() as f64 * COVERED).ceil() as usize;
            let within = |limit: f64| values.iter().filter(|v| v.abs() <= limit).count();
            prop_assert!(values.iter().any(|v| v.abs() == reach));
            prop_assert!(within(reach) >= enough);
            let below = values.iter().filter(|v| v.abs() < reach).count();
            prop_assert!(below < enough);
        }

        #[test]
        fn zero_is_neutral(scale in scale()) {
            prop_assert_eq!(scale.colour(0.0), Rgb::NEUTRAL);
            prop_assert_eq!(scale.colour(-0.0), Rgb::NEUTRAL);
        }

        /// From the reach on, a value has the colour of its end.
        #[test]
        fn the_ends_are_at_the_reach_and_beyond(scale in scale(), beyond in 1.0..10.0f64) {
            prop_assume!(scale.reach() > 0.0);
            let value = scale.reach() * beyond;
            prop_assert_eq!(scale.colour(scale.reach()), Rgb::ABOVE);
            prop_assert_eq!(scale.colour(-scale.reach()), Rgb::BELOW);
            prop_assert_eq!(scale.colour(value), Rgb::ABOVE);
            prop_assert_eq!(scale.colour(-value), Rgb::BELOW);
        }

        /// Where the values covered are all zero, any other value is beyond.
        #[test]
        fn without_a_reach_any_other_value_is_at_an_end(value in 1e-9..1e6f64) {
            let scale = Scale::fitting(&[0.0]);
            prop_assert_eq!(scale.colour(value), Rgb::ABOVE);
            prop_assert_eq!(scale.colour(-value), Rgb::BELOW);
        }

        /// A value and its negative are as far from neutral as each other,
        /// each toward its own end.
        #[test]
        fn a_value_and_its_negative_mirror_each_other(scale in scale(), share in 0.0..=1.0f64) {
            prop_assume!(scale.reach() > 0.0);
            let value = scale.reach() * share;
            let share = value / scale.reach();
            prop_assert_eq!(scale.colour(value), Rgb::toward(Rgb::ABOVE, share));
            prop_assert_eq!(scale.colour(-value), Rgb::toward(Rgb::BELOW, share));
        }

        /// Both ends are darker than the neutral colour in every part, so a
        /// larger value never has a lighter part than a smaller one of its
        /// sign.
        #[test]
        fn a_larger_value_is_never_nearer_neutral(
            scale in scale(),
            a in 0.0..2.0f64,
            b in 0.0..2.0f64,
            negative in any::<bool>(),
        ) {
            prop_assume!(scale.reach() > 0.0);
            let sign = if negative { -1.0 } else { 1.0 };
            let (smaller, larger) = if a <= b { (a, b) } else { (b, a) };
            let near = scale.colour(sign * smaller * scale.reach());
            let far = scale.colour(sign * larger * scale.reach());
            for part in 0..3 {
                prop_assert!(far.0[part] <= near.0[part], "{:?} then {:?}", near, far);
            }
        }
    }
}
