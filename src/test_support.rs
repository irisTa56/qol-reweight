//! What the tests of more than one module make their input with.

use std::fmt::Write as _;
use std::path::PathBuf;

use proptest::prelude::*;

use crate::dataset::Dataset;

/// The code of any half grid square, each digit in the range `mesh`'s
/// documentation gives it.
pub(crate) fn valid_code() -> impl Strategy<Value = String> {
    "[0-9]{4}[0-7]{2}[0-9]{2}[1-4]"
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
            writeln!(
                file,
                "{code},00,00000,a prefecture,{},{indicator_code},{name},{value}",
                city_of(code)
            )
            .unwrap();
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
