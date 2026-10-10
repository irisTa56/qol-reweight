//! What the tests of more than one module make their input with.

use std::path::PathBuf;

use proptest::prelude::*;

use crate::dataset::Dataset;

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

/// A file read as the tool reads one: a mesh for each of `codes`, in a
/// municipality named after its code as [`city_of`] has it, and for each of
/// `series`, in that order in the file, its `IndicatorCode`, its name, and a
/// value for each mesh.
pub(crate) fn dataset_of(codes: &[&str], series: &[(&str, &str, &[f64])]) -> Dataset {
    let mut file = String::from(
        "KeyCode,PrefectureCode,CityCode,Prefecture,City,IndicatorCode,Indicator,Value\n",
    );
    for (indicator_code, name, values) in series {
        for (code, value) in codes.iter().zip(*values) {
            file.push_str(&format!(
                "{code},00,00000,a prefecture,{},{indicator_code},{name},{value}\n",
                city_of(code)
            ));
        }
    }
    Dataset::read(file.as_bytes()).expect("a file made to be read")
}

/// The municipality [`dataset_of`] puts the mesh `code` names in: one of its
/// own for each mesh, so that a name tells which mesh it was read from.
pub(crate) fn city_of(code: &str) -> String {
    format!("City {code}")
}

/// A file's worth of meshes with a published total each. It also has one
/// indicator whose value is the same for all of them and whose rows come
/// first, so that neither a mesh's colour nor the file's order tells the
/// total from the indicator by accident.
pub(crate) fn dataset_of_totals(totals: &[(&str, f64)]) -> Dataset {
    let codes: Vec<&str> = totals.iter().map(|(code, _)| *code).collect();
    let totals: Vec<f64> = totals.iter().map(|(_, total)| *total).collect();
    let same = vec![0.25; codes.len()];
    dataset_of(
        &codes,
        &[("A01", "An indicator", &same), ("QOL", "Total", &totals)],
    )
}

/// Every CSV file in the folder `QOL_REWEIGHT_REAL_FILES` names, in the order
/// of their names. The files are real ones, which the repository does not
/// hold, so a test that reads them runs only when asked for, and what it
/// says of a failure names a file, a mesh, or an indicator and never a
/// value.
pub(crate) fn real_files() -> Vec<PathBuf> {
    let folder = std::env::var_os("QOL_REWEIGHT_REAL_FILES")
        .expect("QOL_REWEIGHT_REAL_FILES names the folder of the files");
    let mut files: Vec<_> = std::fs::read_dir(&folder)
        .expect("the folder can be read")
        .map(|entry| entry.expect("an entry of the folder").path())
        .filter(|path| path.extension().is_some_and(|ending| ending == "csv"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "the folder holds no CSV file");
    files
}
