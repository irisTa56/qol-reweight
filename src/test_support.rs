//! What the tests of more than one module make their input with.

use proptest::prelude::*;

/// The digits of a half grid square code, named as `mesh`'s documentation
/// names them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Digits {
    pub(crate) pp: u32,
    pub(crate) uu: u32,
    pub(crate) q: u32,
    pub(crate) v: u32,
    pub(crate) r: u32,
    pub(crate) w: u32,
    pub(crate) m: u32,
}

impl Digits {
    pub(crate) fn code(self) -> String {
        let Self {
            pp,
            uu,
            q,
            v,
            r,
            w,
            m,
        } = self;
        format!("{pp:02}{uu:02}{q}{v}{r}{w}{m}")
    }
}

/// Any nine digits, with the second-level digits and the quarter drawn from
/// the ranges given.
pub(crate) fn digits(
    second_level: impl Strategy<Value = (u32, u32)>,
    quarter: impl Strategy<Value = u32>,
) -> impl Strategy<Value = Digits> {
    (
        0..100u32,
        0..100u32,
        second_level,
        0..10u32,
        0..10u32,
        quarter,
    )
        .prop_map(|(pp, uu, (q, v), r, w, m)| Digits {
            pp,
            uu,
            q,
            v,
            r,
            w,
            m,
        })
}

/// The digits of any half grid square.
pub(crate) fn valid_digits() -> impl Strategy<Value = Digits> {
    digits((0..8u32, 0..8u32), 1..=4u32)
}

/// The code of any half grid square.
pub(crate) fn valid_code() -> impl Strategy<Value = String> {
    valid_digits().prop_map(Digits::code)
}
