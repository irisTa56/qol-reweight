use std::env;
use std::path::Path;
use std::process::ExitCode;

use dataset::Dataset;

// Until the meshes are drawn, a file is read only to be counted.
#[cfg_attr(not(test), expect(dead_code, reason = "nothing draws a mesh yet"))]
mod dataset;
#[cfg_attr(not(test), expect(dead_code, reason = "nothing draws a mesh yet"))]
mod mesh;
#[cfg(test)]
mod test_support;

/// What the tool is started with, and where that comes from.
const USAGE: &str = "\
Usage: qol-reweight <FILE>

FILE is one of the CSV files of Urban QOL data that the MLIT Data Platform
publishes, one for each prefecture and each metropolitan area, such as
QOL_23_Aichi.csv. Download it from the platform, starting from its page on
the data:
https://data-platform.mlit.go.jp/#/Page?id=dataintro01";

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    // One file: a second argument would go unread without a word.
    let (Some(path), None) = (arguments.next(), arguments.next()) else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    let path = Path::new(&path);
    match Dataset::open(path) {
        Ok(dataset) => {
            println!(
                "Read {} meshes and {} indicators.",
                dataset.meshes().len(),
                dataset.series().len() - 1
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "{} cannot be read as Urban QOL data: {error}\n\n{USAGE}",
                path.display()
            );
            ExitCode::FAILURE
        }
    }
}
