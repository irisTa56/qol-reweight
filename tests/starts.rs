//! Starts of the tool that must end without a map: it says which file it
//! expects and where that file comes from, and exits. One start with a file it
//! can read stands beside them, so that refusing everything does not pass.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn start(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qol-reweight"))
        .args(arguments)
        .output()
        .expect("the tool starts")
}

fn assert_asks_for_the_file(output: &Output) {
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "it exited as if it had worked");
    assert!(output.stdout.is_empty(), "it printed a result");
    assert!(said.contains("Usage: qol-reweight <FILE>"), "{said}");
    assert!(said.contains("CSV files of Urban QOL data"), "{said}");
    assert!(said.contains("https://data-platform.mlit.go.jp/"), "{said}");
}

#[test]
fn started_without_a_path() {
    assert_asks_for_the_file(&start(&[]));
}

#[test]
fn started_with_a_path_that_does_not_exist() {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("no-such-file.csv");
    let output = start(&[path.to_str().unwrap()]);
    assert_asks_for_the_file(&output);
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("no-such-file.csv cannot be read"), "{said}");
}

#[test]
fn started_with_a_file_that_is_not_urban_qol_data() {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("not-qol.csv");
    fs::write(&path, "name,height\ntree,12\n").unwrap();
    let output = start(&[path.to_str().unwrap()]);
    assert_asks_for_the_file(&output);
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("it has no `KeyCode` column"), "{said}");
}

#[test]
fn started_with_two_paths() {
    assert_asks_for_the_file(&start(&["one.csv", "two.csv"]));
}

#[test]
fn started_with_a_folder() {
    let output = start(&[env!("CARGO_TARGET_TMPDIR")]);
    assert_asks_for_the_file(&output);
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("it could not be opened or read"), "{said}");
}

#[test]
fn started_with_a_file_it_can_read() {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("made-up.csv");
    let contents = "KeyCode,PrefectureCode,CityCode,Prefecture,City,IndicatorCode,Indicator,Value\n\
                    543823431,00,00000,a prefecture,East,QOL,Total,1.5\n\
                    543823431,00,00000,a prefecture,East,A01,Stations,1.5\n";
    fs::write(&path, contents).unwrap();
    let output = start(&[path.to_str().unwrap()]);
    assert!(output.status.success(), "it refused a file it can read");
    assert!(output.stderr.is_empty(), "it complained");
}
