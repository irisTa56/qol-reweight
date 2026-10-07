//! The tool's window: the map, and under it the statements of its sources.

use eframe::egui::{CentralPanel, Context, Frame, Margin, Panel, Ui, ViewportBuilder};
use eframe::{App, NativeOptions};
use walkers::{HttpTiles, Map, MapMemory, Position, Tiles, lat_lon};

use crate::basemap::{self, PaleMap};
use crate::dataset::{self, Dataset};
use crate::font::JapaneseFont;
use crate::mesh::Point;

const TITLE: &str = "QOL Reweight";

/// The window's size when it opens, in points.
const SIZE: [f32; 2] = [1280.0, 800.0];

/// The zoom level the map opens at, which shows a city and what lies around
/// it.
const FIRST_ZOOM: f64 = 10.0;

/// What the screen puts before the statement of the data's source, and before
/// that of the base map's.
const DATA_LABEL: &str = include_str!("../assets/data-source-label.txt").trim_ascii_end();
const BASE_MAP_LABEL: &str = include_str!("../assets/base-map-source-label.txt").trim_ascii_end();

/// The space around the statements of the sources, in points: as much above
/// the first and below the last as there is between the two.
const SOURCES_MARGIN: Margin = Margin::symmetric(8, 10);

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
        let options = NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(SIZE),
            ..NativeOptions::default()
        };
        eframe::run_native(
            TITLE,
            options,
            Box::new(move |creation| {
                let context = &creation.egui_ctx;
                let tiles = HttpTiles::new(PaleMap, context.clone());
                Ok(Box::new(Self::new(context, font, centre, Some(tiles))))
            }),
        )
    }

    /// The window as it opens in `context`: `font` draws its Japanese text, and
    /// its map is centred on `centre`.
    fn new(context: &Context, font: JapaneseFont, centre: Point, tiles: Option<HttpTiles>) -> Self {
        context.set_fonts(font.before_the_defaults());
        let mut memory = MapMemory::default();
        memory
            .set_zoom(FIRST_ZOOM)
            .expect("the first zoom level is one the map has");
        Self {
            centre: lat_lon(centre.latitude, centre.longitude),
            tiles,
            memory,
        }
    }

    /// The map, with the statements of its sources: no frame has the one
    /// without the others.
    fn show(&mut self, ui: &mut Ui) {
        Panel::bottom("sources")
            .frame(Frame::side_top_panel(ui.style()).inner_margin(SOURCES_MARGIN))
            .show(ui, Self::state_the_sources);
        CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
            let tiles = self.tiles.as_mut().map(|tiles| tiles as &mut dyn Tiles);
            ui.add(Map::new(tiles, &mut self.memory, self.centre));
        });
    }

    /// Each statement after a label that says what it is the source of.
    fn state_the_sources(ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(DATA_LABEL);
            ui.label(dataset::SOURCE);
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(BASE_MAP_LABEL);
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
    use eframe::egui::{FontDefinitions, FontId, OutputCommand};
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable as _;

    use super::*;

    const CENTRE: Point = Point {
        latitude: 35.0,
        longitude: 137.0,
    };

    /// The window as the tool makes it, but for its tiles: it has none, so
    /// nothing is fetched. A harness has no context to make the window in
    /// until it is itself made, so it starts without one.
    fn window() -> Harness<'static, Option<Window>> {
        let show = |ui: &mut Ui, window: &mut Option<Window>| {
            if let Some(window) = window {
                window.show(ui);
            }
        };
        let mut harness = Harness::new_ui_state(show, None);
        let font = JapaneseFont::installed().expect("macOS has the font");
        let window = Window::new(&harness.ctx, font, CENTRE, None);
        *harness.state_mut() = Some(window);
        // The harness took its size from a frame with no window in it.
        harness.fit_contents();
        harness.run();
        harness
    }

    #[test]
    fn the_sources_are_stated() {
        let window = window();
        window.get_by_label(DATA_LABEL);
        window.get_by_label(dataset::SOURCE);
        window.get_by_label(BASE_MAP_LABEL);
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

    /// The window's own context has the fonts, in the order the font gives
    /// them.
    #[test]
    fn the_window_draws_its_text_in_the_font() {
        let window = window();
        let font = JapaneseFont::installed().expect("macOS has the font");
        let families = window
            .ctx
            .fonts(|fonts| fonts.definitions().families.clone());
        assert_eq!(families, font.before_the_defaults().families);
    }

    /// Which characters the fonts lack, with egui's own put first or not at
    /// all. egui tells a missing character by the font that draws the mark
    /// for one, which is the first font, so with the Japanese font first it
    /// would report every character of that font as missing.
    fn lacks(text: &str, with_the_font: bool) -> bool {
        let mut fonts = FontDefinitions::default();
        if with_the_font {
            fonts = JapaneseFont::installed()
                .expect("macOS has the font")
                .before_the_defaults();
            for names in fonts.families.values_mut() {
                names.rotate_left(1);
            }
        }
        let mut harness = Harness::new_ui(|_| {});
        harness.ctx.set_fonts(fonts);
        harness.run();
        harness
            .ctx
            .fonts_mut(|fonts| !fonts.has_glyphs(&FontId::default(), text))
    }

    /// Everything the screen states is drawn as written, with no character
    /// left to the mark that stands for a missing one.
    #[test]
    fn the_font_has_every_character_of_the_statements() {
        let stated = [
            DATA_LABEL,
            dataset::SOURCE,
            BASE_MAP_LABEL,
            basemap::SOURCE,
            basemap::SHORELINE_CREDIT,
        ];
        for text in stated {
            assert!(!lacks(text, true), "{text}");
        }
    }

    /// The check above can fail: egui's own fonts lack the Japanese ones.
    #[test]
    fn without_the_font_the_statements_cannot_be_drawn() {
        for text in [DATA_LABEL, dataset::SOURCE, BASE_MAP_LABEL, basemap::SOURCE] {
            assert!(lacks(text, false), "{text}");
        }
    }
}
