//! The tool's window: the map, and under it the statements of its sources.

use eframe::egui::{CentralPanel, Frame, Panel, Ui, ViewportBuilder};
use eframe::{App, NativeOptions};
use walkers::{HttpTiles, Map, MapMemory, Position, Tiles, lat_lon};

use crate::basemap::{self, PaleMap};
use crate::dataset::{self, Dataset};
use crate::font::JapaneseFont;

const TITLE: &str = "QOL Reweight";

/// The window's size when it opens, in points.
const SIZE: [f32; 2] = [1280.0, 800.0];

/// The zoom level the map opens at, which shows a city and what lies around
/// it.
const FIRST_ZOOM: f64 = 10.0;

pub(crate) struct Window {
    /// Where the map is centred until its user moves it.
    centre: Position,
    /// The base map's tiles, or none where nothing may be fetched.
    tiles: Option<HttpTiles>,
    memory: MapMemory,
}

impl Window {
    /// Opens the window on `dataset`, and returns when it is closed.
    pub(crate) fn open(dataset: &Dataset, font: JapaneseFont) -> eframe::Result {
        let centre = dataset.centre();
        let centre = lat_lon(centre.latitude, centre.longitude);
        let options = NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(SIZE),
            ..NativeOptions::default()
        };
        eframe::run_native(
            TITLE,
            options,
            Box::new(move |creation| {
                let context = &creation.egui_ctx;
                context.set_fonts(font.after_the_defaults());
                let tiles = HttpTiles::new(PaleMap, context.clone());
                Ok(Box::new(Self::new(centre, Some(tiles))))
            }),
        )
    }

    fn new(centre: Position, tiles: Option<HttpTiles>) -> Self {
        let mut memory = MapMemory::default();
        memory
            .set_zoom(FIRST_ZOOM)
            .expect("the first zoom level is one the map has");
        Self {
            centre,
            tiles,
            memory,
        }
    }

    /// The map, with the statements of its sources: no frame has the one
    /// without the others.
    fn show(&mut self, ui: &mut Ui) {
        Panel::bottom("sources").show(ui, Self::state_the_sources);
        CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
            let tiles = self.tiles.as_mut().map(|tiles| tiles as &mut dyn Tiles);
            ui.add(Map::new(tiles, &mut self.memory, self.centre));
        });
    }

    fn state_the_sources(ui: &mut Ui) {
        ui.label(dataset::SOURCE);
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(basemap::SOURCE, basemap::TILE_LIST);
            ui.small(basemap::SHORELINE_CREDIT);
        });
    }
}

impl App for Window {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

/// These drive the window without showing it, on macOS alone: elsewhere the
/// tool has no font to state the sources in, and shows no map.
#[cfg(all(test, target_os = "macos"))]
mod tests {
    use eframe::egui::{FontId, OutputCommand};
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable as _;

    use super::*;

    /// The window with no tiles to fetch, in the font the tool shows it in.
    fn window() -> Harness<'static, Window> {
        let window = Window::new(lat_lon(35.0, 137.0), None);
        let mut harness = Harness::new_ui_state(|ui, window: &mut Window| window.show(ui), window);
        let font = JapaneseFont::installed().expect("macOS has the font");
        harness.ctx.set_fonts(font.after_the_defaults());
        harness.run();
        harness
    }

    #[test]
    fn the_sources_are_stated() {
        let window = window();
        window.get_by_label(dataset::SOURCE);
        window.get_by_label(basemap::SOURCE);
        window.get_by_label(basemap::SHORELINE_CREDIT);
    }

    #[test]
    fn the_base_maps_source_links_to_the_list_of_tiles() {
        let mut window = window();
        window.get_by_label(basemap::SOURCE).click();
        // The click reaches the link in one of the next frames, and each
        // frame's output replaces the last, so each is read.
        let mut opened = Vec::new();
        for _ in 0..3 {
            window.step();
            let commands = &window.output().platform_output.commands;
            opened.extend(commands.iter().filter_map(|command| match command {
                OutputCommand::OpenUrl(open) => Some(open.url.clone()),
                _ => None,
            }));
        }
        assert_eq!(opened, [basemap::TILE_LIST]);
    }

    /// A statement is drawn in the text it is written in, with no character
    /// left to the mark that stands for a missing one.
    #[test]
    fn the_font_has_every_character_of_the_statements() {
        let window = window();
        let drawn = |text| {
            window
                .ctx
                .fonts_mut(|fonts| fonts.has_glyphs(&FontId::default(), text))
        };
        assert!(drawn(dataset::SOURCE));
        assert!(drawn(basemap::SOURCE));
        assert!(drawn(basemap::SHORELINE_CREDIT));
    }

    /// The check above can fail: egui's own fonts lack those characters.
    #[test]
    fn without_the_font_the_statements_cannot_be_drawn() {
        let harness = Harness::new_ui(|ui| {
            ui.label(dataset::SOURCE);
        });
        let drawn = |text| {
            harness
                .ctx
                .fonts_mut(|fonts| fonts.has_glyphs(&FontId::default(), text))
        };
        assert!(!drawn(dataset::SOURCE));
        assert!(!drawn(basemap::SOURCE));
    }
}
