//! Half grid squares, the 500 m meshes the Urban QOL data is published on.
//!
//! The code of a half grid square (2分の1地域メッシュ) is nine digits,
//! `ppuuqvrwm`, defined by the Statistics Bureau of Japan in
//! [地域メッシュ統計の特質・沿革](https://www.stat.go.jp/data/mesh/pdf/gaiyo1.pdf),
//! pp. 6-12:
//!
//! - `pp` and `uu` name a first-level square, 40′ of latitude by 1° of
//!   longitude: `pp` is its southern latitude times 1.5, and `uu` its western
//!   longitude less 100.
//! - `q` and `v`, each 0 to 7, count second-level squares north and east
//!   within it, 5′ by 7′30″.
//! - `r` and `w`, each 0 to 9, count third-level squares north and east within
//!   that, 30″ by 45″.
//! - `m` names a quarter of the third-level square, 15″ by 22.5″: 1 is the
//!   south-west one, 2 the south-east, 3 the north-west, and 4 the north-east.

use std::fmt;

/// How many half grid squares one degree of latitude spans, each being 15″.
const ROWS_PER_DEGREE: u32 = 240;
/// How many half grid squares one degree of longitude spans, each being 22.5″.
const COLUMNS_PER_DEGREE: u32 = 160;

/// A half grid square, held as its place among all such squares.
///
/// Both counts are whole numbers, so two squares that share an edge compute
/// that edge from the same number and get the same coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HalfMesh {
    /// Squares between the equator and this one's southern edge.
    row: u32,
    /// Squares between 100° east and this one's western edge.
    column: u32,
}

impl HalfMesh {
    /// Reads a nine-digit half grid square code.
    pub fn from_code(code: &str) -> Result<Self, MeshCodeError> {
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
    pub fn south(self) -> f64 {
        f64::from(self.row) / f64::from(ROWS_PER_DEGREE)
    }

    /// Latitude of the northern edge, in degrees.
    pub fn north(self) -> f64 {
        f64::from(self.row + 1) / f64::from(ROWS_PER_DEGREE)
    }

    /// Longitude of the western edge, in degrees east.
    pub fn west(self) -> f64 {
        100.0 + f64::from(self.column) / f64::from(COLUMNS_PER_DEGREE)
    }

    /// Longitude of the eastern edge, in degrees east.
    pub fn east(self) -> f64 {
        100.0 + f64::from(self.column + 1) / f64::from(COLUMNS_PER_DEGREE)
    }
}

/// Why a string is not the code of a half grid square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshCodeError {
    NotNineDigits,
    /// The fifth or sixth digit is 8 or 9, and a first-level square has eight
    /// second-level squares a side.
    NoSuchSecondLevelSquare,
    /// The ninth digit is not 1 to 4. A nine-digit code ending in 5 is a
    /// 2 km square (2倍地域メッシュ), which this tool does not draw.
    NoSuchQuarter,
}

impl fmt::Display for MeshCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotNineDigits => "a half grid square code is nine digits",
            Self::NoSuchSecondLevelSquare => "its fifth and sixth digits go from 0 to 7",
            Self::NoSuchQuarter => "its ninth digit goes from 1 to 4",
        })
    }
}

impl std::error::Error for MeshCodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh(code: &str) -> HalfMesh {
        HalfMesh::from_code(code).unwrap_or_else(|e| panic!("{code}: {e}"))
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{actual} is not {expected}"
        );
    }

    /// The code the Statistics Bureau's tables use as their example: first-level
    /// square 5438 starts at 36°N 138°E, `23` adds 2 × 5′ and 3 × 7′30″, and
    /// `43` adds 4 × 30″ and 3 × 45″.
    #[test]
    fn the_standards_example_lands_where_its_tables_put_it() {
        let m = mesh("543823431");
        assert_close(m.south(), 36.0 + 10.0 / 60.0 + 120.0 / 3600.0);
        assert_close(m.west(), 138.0 + 22.5 / 60.0 + 135.0 / 3600.0);
    }

    #[test]
    fn a_square_is_15_seconds_by_22_5_seconds() {
        let m = mesh("543823431");
        assert_close(m.north() - m.south(), 15.0 / 3600.0);
        assert_close(m.east() - m.west(), 22.5 / 3600.0);
    }

    #[test]
    fn the_ninth_digit_goes_south_west_south_east_north_west_north_east() {
        let [sw, se, nw, ne] = ["1", "2", "3", "4"].map(|m| mesh(&format!("54382343{m}")));
        // South row: 1 then 2, west to east.
        assert_eq!(sw.south(), se.south());
        assert_eq!(sw.east(), se.west());
        // North row: 3 then 4, above them.
        assert_eq!(nw.south(), sw.north());
        assert_eq!(nw.west(), sw.west());
        assert_eq!(ne.south(), se.north());
        assert_eq!(ne.west(), se.west());
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
    fn anything_but_nine_ascii_digits_is_refused() {
        for code in [
            "",
            "54382343",
            "5438234311",
            "54382343a",
            " 543823431",
            "５４３８２３４３１",
            // Nine bytes, which only a check of the digits themselves refuses.
            "５４３",
        ] {
            assert_eq!(
                HalfMesh::from_code(code),
                Err(MeshCodeError::NotNineDigits),
                "{code:?}"
            );
        }
    }

    #[test]
    fn a_second_level_digit_past_7_is_refused() {
        for code in ["543883431", "543828431"] {
            assert_eq!(
                HalfMesh::from_code(code),
                Err(MeshCodeError::NoSuchSecondLevelSquare),
                "{code}"
            );
        }
    }

    #[test]
    fn a_ninth_digit_outside_1_to_4_is_refused() {
        // 5 is the nine-digit code of a 2 km square.
        for code in ["543823430", "543823435", "543823439"] {
            assert_eq!(
                HalfMesh::from_code(code),
                Err(MeshCodeError::NoSuchQuarter),
                "{code}"
            );
        }
    }
}
