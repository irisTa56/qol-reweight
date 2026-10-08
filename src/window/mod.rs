//! The tool's window: the map, beside it what its colours show, and under
//! them the statements of its sources.

use eframe::egui::{CentralPanel, ComboBox, Context, Frame, Margin, Panel, Ui, ViewportBuilder};
use eframe::epaint::text::FontPriority;
use eframe::{App, NativeOptions};
use walkers::{HttpTiles, Map, MapMemory, Position, Tiles};

use crate::basemap::{self, PaleMap};
use crate::dataset::{self, Dataset};
use crate::font::JapaneseFont;
use crate::layer::MeshLayer;
use crate::legend::Legend;
use crate::view::View;

const TITLE: &str = "QOL Reweight";

/// The window's size when it opens, in points.
const SIZE: [f32; 2] = [1280.0, 800.0];

/// What the screen puts before the statement of the data's source, and before
/// that of the base map's.
const DATA_LABEL: &str = include_str!("../../assets/data-source-label.txt").trim_ascii_end();
const BASE_MAP_LABEL: &str =
    include_str!("../../assets/base-map-source-label.txt").trim_ascii_end();

/// What the screen puts over the pull-down of what the colours can show.
const SHOWN_LABEL: &str = include_str!("../../assets/shown-label.txt").trim_ascii_end();

/// How wide the panel with the legend and that pull-down is, in points.
const CHOICES_WIDTH: f32 = 240.0;

/// How high the pull-down opens at most, in points: some twenty choices,
/// past which it scrolls.
const CHOICES_HEIGHT: f32 = 500.0;

/// The space above the legend and under it, in points.
const CHOICES_SPACE: f32 = 8.0;

/// The space around the statements of the sources, in points: as much above
/// the first and below the last as there is between the two.
const SOURCES_MARGIN: Margin = Margin::symmetric(8, 10);

pub(crate) struct Window {
    dataset: Dataset,
    /// Which of the dataset's series the colours show: the total, which
    /// comes first, until another is chosen.
    shown: usize,
    /// Where the map is centred until its user moves it.
    centre: Position,
    /// The base map's tiles, or none where nothing may be fetched.
    tiles: Option<HttpTiles>,
    memory: MapMemory,
    layer: MeshLayer,
}

impl Window {
    /// Opens the window on `dataset`, and returns when it is closed.
    pub(crate) fn open(dataset: Dataset, font: JapaneseFont) -> eframe::Result {
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
                Ok(Box::new(Self::new(context, font, dataset, Some(tiles))))
            }),
        )
    }

    /// The window as it opens in `context`: `font` draws its Japanese text, and
    /// its map is centred on the meshes of `dataset`, each drawn in the
    /// colour of its published total.
    fn new(
        context: &Context,
        font: JapaneseFont,
        dataset: Dataset,
        tiles: Option<HttpTiles>,
    ) -> Self {
        context.add_font(font.into_insert(FontPriority::Highest));
        let view = View::of(dataset.extent());
        let mut memory = MapMemory::default();
        memory
            .set_zoom(view.zoom())
            .expect("the opening zoom level is one the map has");
        let shown = 0;
        let layer = MeshLayer::showing(dataset.meshes(), &dataset.series()[shown]);
        Self {
            dataset,
            shown,
            centre: view.centre(),
            tiles,
            memory,
            layer,
        }
    }

    /// The map, with the statements of its sources: no frame has the one
    /// without the others. Beside the map, what its colours show.
    fn show(&mut self, ui: &mut Ui) {
        Panel::bottom("sources")
            .frame(Frame::side_top_panel(ui.style()).inner_margin(SOURCES_MARGIN))
            .show(ui, Self::state_the_sources);
        Panel::left("shown")
            .resizable(false)
            .default_size(CHOICES_WIDTH)
            .show(ui, |ui| self.choose_what_is_shown(ui));
        CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
            let tiles = self.tiles.as_mut().map(|tiles| tiles as &mut dyn Tiles);
            let layer = &self.layer;
            Map::new(tiles, &mut self.memory, self.centre).show(ui, |ui, _, projector, _| {
                ui.painter().add(layer.shape(projector));
            });
        });
    }

    /// The legend of what is shown, and under it a pull-down to choose from:
    /// the total, then each indicator of the file. A choice colours the
    /// meshes anew.
    fn choose_what_is_shown(&mut self, ui: &mut Ui) {
        ui.add_space(CHOICES_SPACE);
        ui.add(Legend::of(self.layer.paint()));
        ui.add_space(CHOICES_SPACE);
        ui.strong(SHOWN_LABEL);
        let series = self.dataset.series();
        let mut chosen = self.shown;
        // Closed, it takes one line whatever the file holds, and a name too
        // long for that line is cut short.
        ComboBox::from_id_salt("shown")
            .selected_text(series[chosen].name())
            .width(ui.available_width())
            .height(CHOICES_HEIGHT)
            .truncate()
            .show_ui(ui, |ui| {
                for (at, series) in series.iter().enumerate() {
                    ui.selectable_value(&mut chosen, at, series.name());
                }
            });
        if chosen != self.shown {
            self.shown = chosen;
            self.layer = MeshLayer::showing(self.dataset.meshes(), &series[chosen]);
        }
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

#[cfg(all(test, target_os = "macos"))]
mod tests;
