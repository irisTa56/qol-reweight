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
const DATA_LABEL: &str = include_str!("../assets/data-source-label.txt").trim_ascii_end();
const BASE_MAP_LABEL: &str = include_str!("../assets/base-map-source-label.txt").trim_ascii_end();

/// What the screen puts over the pull-down of what the colours can show.
const SHOWN_LABEL: &str = include_str!("../assets/shown-label.txt").trim_ascii_end();

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

/// These drive the window without showing it, on macOS alone: elsewhere the
/// tool has no font to state the sources in, and shows no map.
#[cfg(all(test, target_os = "macos"))]
mod tests {
    use eframe::egui::accesskit::Role;
    use eframe::egui::{FontFamily, FontId, OutputCommand, Rect, Shape};
    use eframe::epaint::Vertex;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT as _, Queryable as _};

    use walkers::{Projector, lat_lon};

    use super::*;
    use crate::test_support::dataset_of;

    /// The meshes of the file the window is opened on.
    const MESHES: [&str; 3] = ["543823431", "543823432", "543823434"];

    /// What the file publishes for those meshes, in its order: the total
    /// comes last, and falls where one indicator rises.
    const FILE: [(&str, &str, &[f64]); 3] = [
        ("A01", "Stations", &[0.25, 0.25, 0.25]),
        ("B02", "Floods", &[-3.0, 0.0, 3.0]),
        ("QOL", "Total", &[2.0, 0.5, -1.0]),
    ];

    /// The window as the tool makes it, but for its tiles: it has none, so
    /// nothing is fetched. A harness has no context to make the window in
    /// until it is itself made, so it starts without one.
    fn window() -> Harness<'static, Option<Window>> {
        window_on(dataset_of(&MESHES, &FILE))
    }

    /// The same, opened on `dataset`.
    fn window_on(dataset: Dataset) -> Harness<'static, Option<Window>> {
        let show = |ui: &mut Ui, window: &mut Option<Window>| {
            if let Some(window) = window {
                window.show(ui);
            }
        };
        let mut harness = Harness::new_ui_state(show, None);
        let font = JapaneseFont::installed().expect("macOS has the font");
        let window = Window::new(&harness.ctx, font, dataset, None);
        *harness.state_mut() = Some(window);
        // The harness took its size from a frame with no window in it.
        harness.fit_contents();
        harness.run();
        harness
    }

    /// What the window draws of the meshes: the map's rectangle, and each
    /// corner of each square with its colour.
    fn meshes_drawn(window: &Harness<'_, Option<Window>>) -> (Rect, Vec<Vertex>) {
        let drawn: Vec<_> = window
            .output()
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                // Four corners a mesh: the legend's bar is triangles too.
                Shape::Mesh(triangles) if triangles.vertices.len() == 4 * MESHES.len() => {
                    Some((clipped.clip_rect, triangles.vertices.clone()))
                }
                _ => None,
            })
            .collect();
        let [drawn] = drawn.as_slice() else {
            panic!("the window draws the meshes {} times", drawn.len());
        };
        drawn.clone()
    }

    /// The meshes lie where the view the map opens with puts their extent.
    #[test]
    fn the_map_opens_centred_on_the_meshes() {
        let (map, corners) = meshes_drawn(&window());
        let places: Vec<_> = corners.iter().map(|corner| corner.pos).collect();
        let squares = Rect::from_points(&places);

        let extent = dataset_of(&MESHES, &FILE).extent();
        let view = View::of(extent);
        let mut memory = MapMemory::default();
        memory.set_zoom(view.zoom()).unwrap();
        let projector = Projector::new(map, &memory, view.centre());
        let south_west = projector.project(lat_lon(extent.south(), extent.west()));
        let north_east = projector.project(lat_lon(extent.north(), extent.east()));
        let expected = Rect::from_two_pos(south_west.to_pos2(), north_east.to_pos2());

        assert!(map.contains_rect(squares), "{squares:?} in {map:?}");
        assert!(
            (squares.center() - map.center()).length() < 0.5,
            "{squares:?}"
        );
        assert!(
            (squares.min - expected.min).length() < 0.5
                && (squares.max - expected.max).length() < 0.5,
            "{squares:?} against {expected:?}"
        );
    }

    /// Which end of the scale the first mesh and the last are drawn at:
    /// each has the value furthest from zero one way.
    fn ends(window: &Harness<'_, Option<Window>>) -> [&'static str; 2] {
        let (_, corners) = meshes_drawn(window);
        [0, corners.len() - 4].map(|corner| {
            let colour = corners[corner].color;
            if colour.r() > colour.b() + 30 {
                "red"
            } else if colour.b() > colour.r() + 30 {
                "blue"
            } else {
                "neither"
            }
        })
    }

    /// The totals of [`FILE`] fall from above zero to below it, which no
    /// indicator of the file does.
    #[test]
    fn the_meshes_have_the_colours_of_their_totals() {
        assert_eq!(ends(&window()), ["blue", "red"]);
    }

    /// Opens the pull-down, so that its choices are on screen.
    fn open_the_choices(window: &mut Harness<'_, Option<Window>>) {
        window.get_by_role(Role::ComboBox).click();
        window.run();
    }

    /// Opens the pull-down and picks the choice named `name` from it.
    fn choose(window: &mut Harness<'_, Option<Window>>, name: &str) {
        open_the_choices(window);
        let choice = window
            .query_all_by_label(name)
            .find(|named| named.accesskit_node().role() != Role::ComboBox)
            .expect("the pull-down has the choice");
        choice.scroll_to_me();
        window.run();
        window
            .query_all_by_label(name)
            .find(|named| named.accesskit_node().role() != Role::ComboBox)
            .expect("the pull-down has the choice")
            .click();
        window.run();
    }

    /// Closed, the pull-down names what is shown, the total at first. Open,
    /// it has the total and then the indicators in the order the file has
    /// them, which is not the order of the file's rows, where the total comes
    /// last.
    #[test]
    fn the_choices_are_the_total_then_the_indicators_of_the_file() {
        let mut window = window();
        window.get_by_label(SHOWN_LABEL);
        let shown = window.get_by_role(Role::ComboBox).accesskit_node().value();
        assert_eq!(shown.as_deref(), Some("Total"));

        open_the_choices(&mut window);
        let mut choices: Vec<_> = ["Total", "Stations", "Floods"]
            .into_iter()
            .flat_map(|name| window.query_all_by_label(name))
            .filter(|named| named.accesskit_node().role() != Role::ComboBox)
            .map(|choice| {
                (
                    choice.rect().top(),
                    choice.accesskit_node().label().unwrap(),
                )
            })
            .collect();
        choices.sort_by(|above, below| above.0.total_cmp(&below.0));
        let names: Vec<_> = choices.into_iter().map(|(_, name)| name).collect();
        assert_eq!(names, ["Total", "Stations", "Floods"]);
    }

    #[test]
    fn choosing_an_indicator_colours_the_meshes_by_it_and_changes_the_legend() {
        let mut window = window();
        // The total reaches 2, and the indicator 3.
        window.get_by_label("+2.0");
        window.get_by_label("-2.0");

        choose(&mut window, "Floods");

        assert_eq!(ends(&window), ["red", "blue"]);
        window.get_by_label("+3.0");
        window.get_by_label("-3.0");
        assert!(window.query_by_label("+2.0").is_none());
        let shown = window.get_by_role(Role::ComboBox).accesskit_node().value();
        assert_eq!(shown.as_deref(), Some("Floods"));
    }

    #[test]
    fn choosing_the_total_again_brings_its_colours_back() {
        let mut window = window();
        choose(&mut window, "Floods");
        choose(&mut window, "Total");
        assert_eq!(ends(&window), ["blue", "red"]);
        window.get_by_label("+2.0");
    }

    /// A file with more indicators than the open pull-down shows at once,
    /// the last of which falls where the total rises: the pull-down scrolls
    /// to it. Closed, the pull-down takes the room it takes for a short file.
    #[test]
    fn the_last_of_many_indicators_can_be_scrolled_to_and_chosen() {
        let same: &[f64] = &[0.25, 0.25, 0.25];
        let many: Vec<(String, String)> = (0..40)
            .map(|at| (format!("X{at:02}"), format!("Indicator {at}")))
            .collect();
        let mut file: Vec<(&str, &str, &[f64])> = many
            .iter()
            .map(|(code, name)| (code.as_str(), name.as_str(), same))
            .collect();
        file.push(("Y00", "Last", &[3.0, 0.0, -3.0]));
        file.push(("QOL", "Total", &[-2.0, 0.5, 1.0]));

        let closed = window().get_by_role(Role::ComboBox).rect();
        let mut window = window_on(dataset_of(&MESHES, &file));
        assert_eq!(
            window.get_by_role(Role::ComboBox).rect().size(),
            closed.size()
        );
        assert_eq!(ends(&window), ["red", "blue"]);

        choose(&mut window, "Last");
        assert_eq!(ends(&window), ["blue", "red"]);
    }

    /// The bar is over its marks, with the mark of the lower end under the
    /// end where the bar is red and that of the upper end where it is blue.
    #[test]
    fn the_legend_puts_each_mark_under_the_colour_it_stands_for() {
        let window = window();
        let bars: Vec<_> = window
            .output()
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Mesh(triangles) if triangles.vertices.len() != 4 * MESHES.len() => {
                    Some(triangles.vertices.clone())
                }
                _ => None,
            })
            .collect();
        let [bar] = bars.as_slice() else {
            panic!("the window draws {} bars of colours", bars.len());
        };
        // The first four corners are the bar's ground; a step of colour
        // follows for each four after them, from the left.
        let (left, right) = (bar[4], bar[bar.len() - 1]);
        let redder = |corner: Vertex| i16::from(corner.color.r()) - i16::from(corner.color.b());
        let lowest = window.get_by_label("-2.0").rect();
        let middle = window.get_by_label("0").rect();
        let highest = window.get_by_label("+2.0").rect();

        assert!(
            redder(left) > 20 && redder(right) < -20,
            "{left:?} {right:?}"
        );
        assert!(left.pos.x < right.pos.x, "{left:?} {right:?}");
        assert!(lowest.center().x < middle.center().x && middle.center().x < highest.center().x);
        assert!(
            (lowest.left() - left.pos.x).abs() < 8.0,
            "{lowest:?} {left:?}"
        );
        assert!(
            (highest.right() - right.pos.x).abs() < 8.0,
            "{highest:?} {right:?}"
        );
        assert!(bar.iter().all(|corner| corner.pos.y <= lowest.top()));
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

    /// The window's own context has the font, before egui's own.
    #[test]
    fn the_window_draws_its_text_in_the_font() {
        let window = window();
        let font = JapaneseFont::installed().expect("macOS has the font");
        let name = font.into_insert(FontPriority::Highest).name;
        let families = window
            .ctx
            .fonts(|fonts| fonts.definitions().families.clone());
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            assert_eq!(families[&family][0], name);
            assert!(families[&family].len() > 1, "egui's fonts are gone");
        }
    }

    /// Whether the fonts lack a character of `text`, with the Japanese font
    /// after egui's own or not there at all. egui tells a missing character
    /// by the font that draws the mark for one, which is the first font, so
    /// with the Japanese font first it would report every character of that
    /// font as missing.
    fn lacks(text: &str, with_the_font: bool) -> bool {
        let mut harness = Harness::new_ui(|_| {});
        if with_the_font {
            let font = JapaneseFont::installed().expect("macOS has the font");
            harness.ctx.add_font(font.into_insert(FontPriority::Lowest));
        }
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
            SHOWN_LABEL,
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
