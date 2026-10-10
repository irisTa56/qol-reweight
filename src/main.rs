use std::env;
use std::process::ExitCode;

use dataset::Dataset;
use font::JapaneseFont;
use window::Window;

mod asset;
mod basemap;
mod dataset;
mod font;
mod layer;
mod legend;
mod mesh;
mod multipliers;
mod paint;
mod readout;
mod scale;
#[cfg(test)]
mod test_support;
mod view;
mod window;

/// What the tool is started with, and where that comes from.
const USAGE: &str = "\
Usage: qol-reweight <FILE>

FILE is one of the CSV files of Urban QOL data that the MLIT Data Platform publishes,
one for each prefecture and each metropolitan area, such as QOL_23_Aichi.csv.
Download it from the platform, starting from its page on the data:
https://data-platform.mlit.go.jp/#/Page?id=dataintro01";

const NO_MAP_WITHOUT_THE_FONT: &str = "\
The map is not shown, because the sources of the data and of the base map
cannot be stated on screen without a font for Japanese text";

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    // One file: a second argument would go unread without a word.
    let (Some(path), None) = (arguments.next(), arguments.next()) else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    let dataset = match Dataset::open(&path) {
        Ok(dataset) => dataset,
        Err(error) => {
            eprintln!(
                "{} cannot be read as Urban QOL data: {error}\n\n{USAGE}",
                path.display()
            );
            return ExitCode::FAILURE;
        }
    };
    // Before any window: without the font, the map's sources cannot be stated.
    let font = match JapaneseFont::installed() {
        Ok(font) => font,
        Err(error) => {
            eprintln!("{NO_MAP_WITHOUT_THE_FONT}: {error}.");
            return ExitCode::FAILURE;
        }
    };
    match Window::open(dataset, font) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("The window could not be opened: {error}");
            ExitCode::FAILURE
        }
    }
}
