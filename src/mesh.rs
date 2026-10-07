//! Half grid squares, the 500 m meshes the Urban QOL data is published on.
//!
//! The code of a half grid square is nine digits, `ppuuqvrwm`, defined by the
//! Statistics Bureau of Japan in
//! [its outline of grid square statistics](https://www.stat.go.jp/data/mesh/pdf/gaiyo1.pdf),
//! pp. 6-12:
//!
//! - `pp` and `uu` name a first-level square, 40' of latitude by 1 degree of
//!   longitude: `pp` is its southern latitude times 1.5, and `uu` its western
//!   longitude less 100.
//! - `q` and `v`, each 0 to 7, count second-level squares north and east
//!   within it, 5' by 7'30".
//! - `r` and `w`, each 0 to 9, count third-level squares north and east within
//!   that, 30" by 45".
//! - `m` names a quarter of the third-level square, 15" by 22.5": 1 is the
//!   south-west one, 2 the south-east, 3 the north-west, and 4 the north-east.
//!
//! `dev/GLOSSARY.md` gives the Japanese term for each of these names.

use thiserror::Error;

/// How many half grid squares one degree of latitude spans, each being 15".
const ROWS_PER_DEGREE: u32 = 240;
/// How many half grid squares one degree of longitude spans, each being 22.5".
const COLUMNS_PER_DEGREE: u32 = 160;

/// A place on the ground, in degrees north of the equator and east of
/// Greenwich.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Point {
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
}

/// A half grid square, held as its place among all such squares.
///
/// Both counts are whole numbers, so two squares that share an edge compute
/// that edge from the same number and get the same coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct HalfMesh {
    /// Squares between the equator and this one's southern edge.
    row: u32,
    /// Squares between 100 degrees east and this one's western edge.
    column: u32,
}

impl HalfMesh {
    /// Reads a nine-digit half grid square code.
    pub(crate) fn from_code(code: &str) -> Result<Self, MeshCodeError> {
        if !code.bytes().all(|b| b.is_ascii_digit()) {
            return Err(MeshCodeError::NotNineDigits);
        }
        let digits: Vec<u32> = code.bytes().map(|b| u32::from(b - b'0')).collect();
        let &[p1, p2, u1, u2, q, v, r, w, m] = digits.as_slice() else {
            return Err(MeshCodeError::NotNineDigits);
        };
        if q > 7 || v > 7 {
            return Err(MeshCodeError::NoSuchSecondLevelSquare);
        }
        if !(1..=4).contains(&m) {
            return Err(MeshCodeError::NoSuchQuarter);
        }
        // A first-level square is 160 rows high, a second-level one 20, and a
        // third-level one 2; the same counts hold for columns.
        Ok(Self {
            row: (p1 * 10 + p2) * 160 + q * 20 + r * 2 + (m - 1) / 2,
            column: (u1 * 10 + u2) * 160 + v * 20 + w * 2 + (m - 1) % 2,
        })
    }

    /// Latitude of the southern edge, in degrees.
    pub(crate) fn south(self) -> f64 {
        f64::from(self.row) / f64::from(ROWS_PER_DEGREE)
    }

    /// Latitude of the northern edge, in degrees.
    pub(crate) fn north(self) -> f64 {
        f64::from(self.row + 1) / f64::from(ROWS_PER_DEGREE)
    }

    /// Longitude of the western edge, in degrees east.
    pub(crate) fn west(self) -> f64 {
        100.0 + f64::from(self.column) / f64::from(COLUMNS_PER_DEGREE)
    }

    /// Longitude of the eastern edge, in degrees east.
    pub(crate) fn east(self) -> f64 {
        100.0 + f64::from(self.column + 1) / f64::from(COLUMNS_PER_DEGREE)
    }
}

/// Why a string is not the code of a half grid square.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub(crate) enum MeshCodeError {
    #[error("a half grid square code is nine digits")]
    NotNineDigits,
    /// The fifth or sixth digit is 8 or 9, and a first-level square has eight
    /// second-level squares a side.
    #[error("its fifth and sixth digits go from 0 to 7")]
    NoSuchSecondLevelSquare,
    /// The ninth digit is not 1 to 4. A nine-digit code ending in 5 is a
    /// 2 km square, which this tool does not draw.
    #[error("its ninth digit goes from 1 to 4")]
    NoSuchQuarter,
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::test_support::{Digits, digits, valid_digits};

    fn mesh(code: &str) -> HalfMesh {
        HalfMesh::from_code(code).unwrap_or_else(|e| panic!("{code}: {e}"))
    }

    fn close(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    /// The same digits in their full-width forms, three bytes each, which
    /// the source keeps out of its own text.
    fn full_width(ascii_digits: &str) -> String {
        ascii_digits
            .chars()
            .map(|c| char::from_u32(0xFF10 + c.to_digit(10).unwrap()).unwrap())
            .collect()
    }

    /// The code the Statistics Bureau's tables use as their example:
    /// first-level square 5438 starts at 36 degrees north and 138 degrees
    /// east, `23` adds 2 x 5' and 3 x 7'30", and `43` adds 4 x 30" and 3 x 45".
    #[test]
    fn the_standards_example_lands_where_its_tables_put_it() {
        let m = mesh("543823431");
        assert!(close(m.south(), 36.0 + 10.0 / 60.0 + 120.0 / 3600.0));
        assert!(close(m.west(), 138.0 + 22.5 / 60.0 + 135.0 / 3600.0));
    }

    #[test]
    fn squares_meet_across_every_level_of_the_code() {
        // Third level: w 3 -> 4.
        assert_eq!(mesh("543823432").east(), mesh("543823441").west());
        // Second level: v 3 -> 4 as w wraps from 9 to 0.
        assert_eq!(mesh("543823492").east(), mesh("543824401").west());
        // First level: uu 38 -> 39 as v wraps from 7 to 0.
        assert_eq!(mesh("543827492").east(), mesh("543920401").west());
        // The same northwards: r 3 -> 4, q 2 -> 3, pp 54 -> 55.
        assert_eq!(mesh("543823333").north(), mesh("543823431").south());
        assert_eq!(mesh("543823933").north(), mesh("543833031").south());
        assert_eq!(mesh("543873933").north(), mesh("553803031").south());
    }

    #[test]
    fn full_width_digits_are_refused() {
        for code in [
            // Nine characters, 27 bytes.
            full_width("543823431"),
            // Nine bytes, which only a check of the digits themselves refuses.
            full_width("543"),
        ] {
            assert_eq!(
                HalfMesh::from_code(&code),
                Err(MeshCodeError::NotNineDigits),
                "{code:?}"
            );
        }
    }

    proptest! {
        /// Each digit moves the south-west corner by what the standard gives
        /// it, worked out here in degrees rather than in rows and columns.
        #[test]
        fn a_code_lands_where_its_digits_put_it(d in valid_digits()) {
            let m = mesh(&d.code());
            let south = f64::from(d.pp) / 1.5
                + f64::from(d.q) * 5.0 / 60.0
                + f64::from(d.r) * 30.0 / 3600.0
                + f64::from((d.m - 1) / 2) * 15.0 / 3600.0;
            let west = 100.0
                + f64::from(d.uu)
                + f64::from(d.v) * 7.5 / 60.0
                + f64::from(d.w) * 45.0 / 3600.0
                + f64::from((d.m - 1) % 2) * 22.5 / 3600.0;
            prop_assert!(close(m.south(), south), "south {} is not {south}", m.south());
            prop_assert!(close(m.west(), west), "west {} is not {west}", m.west());
        }

        #[test]
        fn a_square_is_15_seconds_by_22_5_seconds(d in valid_digits()) {
            let m = mesh(&d.code());
            prop_assert!(close(m.north() - m.south(), 15.0 / 3600.0));
            prop_assert!(close(m.east() - m.west(), 22.5 / 3600.0));
        }

        /// The four quarters of a third-level square tile it: 1 and 2 along
        /// the south, 3 and 4 above them.
        #[test]
        fn the_quarters_of_a_square_meet(d in valid_digits()) {
            let [sw, se, nw, ne] = [1, 2, 3, 4].map(|m| mesh(&Digits { m, ..d }.code()));
            prop_assert_eq!(sw.south(), se.south());
            prop_assert_eq!(sw.east(), se.west());
            prop_assert_eq!(nw.south(), sw.north());
            prop_assert_eq!(nw.west(), sw.west());
            prop_assert_eq!(ne.south(), se.north());
            prop_assert_eq!(ne.west(), se.west());
        }

        /// Text in any script, digits of other scripts, and ASCII digits that
        /// are too few or too many.
        #[test]
        fn anything_but_nine_ascii_digits_is_refused(
            code in r"\PC{0,12}|\p{Nd}{1,12}|[0-9]{0,8}|[0-9]{10,12}",
        ) {
            prop_assume!(!(code.len() == 9 && code.bytes().all(|b| b.is_ascii_digit())));
            prop_assert_eq!(HalfMesh::from_code(&code), Err(MeshCodeError::NotNineDigits));
        }

        #[test]
        fn a_second_level_digit_past_7_is_refused(
            d in digits(
                prop_oneof![(8..10u32, 0..10u32), (0..10u32, 8..10u32)],
                1..=4u32,
            ),
        ) {
            prop_assert_eq!(
                HalfMesh::from_code(&d.code()),
                Err(MeshCodeError::NoSuchSecondLevelSquare)
            );
        }

        /// 5 among them, the ninth digit of a 2 km square's code.
        #[test]
        fn a_ninth_digit_outside_1_to_4_is_refused(
            d in digits((0..8u32, 0..8u32), prop_oneof![Just(0u32), 5..10u32]),
        ) {
            prop_assert_eq!(
                HalfMesh::from_code(&d.code()),
                Err(MeshCodeError::NoSuchQuarter)
            );
        }
    }
}
