//! The tool's window: the map, beside it what its colours show and what the
//! file says of the mesh pointed at, and under them the statements of its
//! sources.

use eframe::egui::{
    CentralPanel, ComboBox, Context, Frame, Margin, Panel, Slider, Ui, ViewportBuilder,
};
use eframe::epaint::text::FontPriority;
use eframe::{App, NativeOptions};
use walkers::{HttpTiles, Map, MapMemory, Position, Tiles};

use crate::asset;
use crate::basemap::{self, PaleMap};
use crate::dataset::{self, Dataset};
use crate::font::JapaneseFont;
use crate::layer::MeshLayer;
use crate::legend::Legend;
use crate::mesh::HalfMesh;
use crate::paint::OPENING_OPACITY;
use crate::readout::Readout;
use crate::view::View;

const TITLE: &str = "QOL Reweight";

/// The window's size when it opens, in points.
const SIZE: [f32; 2] = [1280.0, 800.0];

/// What the screen puts before the statement of the data's source, and before
/// that of the base map's.
const DATA_LABEL: &str = asset::text!("data-source-label.txt");
const BASE_MAP_LABEL: &str = asset::text!("base-map-source-label.txt");

/// What the screen puts over the pull-down of what the colours can show.
const SHOWN_LABEL: &str = asset::text!("shown-label.txt");

/// What the screen puts over the slider that sets how much of the base map
/// the meshes cover.
const OPACITY_LABEL: &str = asset::text!("opacity-label.txt");

/// How wide the panel with the legend and that pull-down is, in points.
const CHOICES_WIDTH: f32 = 240.0;

/// How high the pull-down opens at most, in points: some twenty choices,
/// past which it scrolls.
const CHOICES_HEIGHT: f32 = 500.0;

/// The space above the legend, under it, and over the readout, in points.
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
    /// How much of the base map the meshes cover, of 255.
    opacity: u8,
    /// Which of the dataset's meshes the pointer is on, if it is on one.
    pointed: Option<usize>,
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
        let opacity = OPENING_OPACITY;
        let layer = MeshLayer::showing(dataset.meshes(), &dataset.series()[shown], opacity);
        Self {
            dataset,
            shown,
            centre: view.centre(),
            tiles,
            memory,
            layer,
            opacity,
            pointed: None,
        }
    }

    /// The map, with the statements of its sources: no frame has the one
    /// without the others. Beside the map, what its colours show, and what
    /// the file says of the mesh the pointer is on.
    fn show(&mut self, ui: &mut Ui) {
        Panel::bottom("sources")
            .frame(Frame::side_top_panel(ui.style()).inner_margin(SOURCES_MARGIN))
            .show(ui, Self::state_the_sources);
        Panel::left("shown")
            .resizable(false)
            .default_size(CHOICES_WIDTH)
            .show(ui, |ui| {
                self.choose_what_is_shown(ui);
                ui.add_space(CHOICES_SPACE);
                ui.add(self.readout());
            });
        CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
            let tiles = self.tiles.as_mut().map(|tiles| tiles as &mut dyn Tiles);
            let layer = &self.layer;
            let pointer = Map::new(tiles, &mut self.memory, self.centre)
                .show(ui, |ui, map, projector, _| {
                    ui.painter().add(layer.shape(projector));
                    // A map being dragged counts as pointed at wherever
                    // the pointer has gone, so the pointer's place is asked
                    // for as well.
                    let pointer = map.hover_pos().filter(|at| map.rect.contains(*at))?;
                    Some(projector.unproject(pointer.to_vec2()))
                })
                .inner;
            let pointed = pointer
                .and_then(|pointer| HalfMesh::holding(pointer.y(), pointer.x()))
                .and_then(|square| self.dataset.mesh_at(square));
            // The readout was drawn before the map was, so it is drawn again.
            if pointed != self.pointed {
                self.pointed = pointed;
                ui.ctx().request_repaint();
            }
        });
    }

    /// What the file says of the mesh the pointer is on: its value is the one
    /// the colours show.
    fn readout(&self) -> Readout<'_> {
        let values = self.dataset.series()[self.shown].values();
        let pointed = self
            .pointed
            .map(|at| (&self.dataset.meshes()[at], values[at]));
        Readout::of(pointed)
    }

    /// The legend of what is shown, under it a pull-down to choose from, the
    /// total, then each indicator of the file, and under that a slider for
    /// how much of the base map the meshes cover, from none of it to all. A
    /// choice colours the meshes anew, and so does a move of the slider.
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
        ui.add_space(CHOICES_SPACE);
        ui.strong(OPACITY_LABEL);
        let mut opacity = self.opacity;
        ui.spacing_mut().slider_width = ui.available_width();
        ui.add(Slider::new(&mut opacity, 0..=u8::MAX).show_value(false));
        if chosen != self.shown || opacity != self.opacity {
            self.shown = chosen;
            self.opacity = opacity;
            self.layer = MeshLayer::showing(self.dataset.meshes(), &series[chosen], opacity);
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
    use crate::readout::{CODE_LABEL, MUNICIPALITY_LABEL, VALUE_LABEL};
    use crate::test_support::{city_of, dataset_of};

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
        window.get_by_label("+2");
        window.get_by_label("-2");

        choose(&mut window, "Floods");

        assert_eq!(ends(&window), ["red", "blue"]);
        window.get_by_label("+3");
        window.get_by_label("-3");
        assert!(window.query_by_label("+2").is_none());
        let shown = window.get_by_role(Role::ComboBox).accesskit_node().value();
        assert_eq!(shown.as_deref(), Some("Floods"));
    }

    #[test]
    fn choosing_the_total_again_brings_its_colours_back() {
        let mut window = window();
        choose(&mut window, "Floods");
        choose(&mut window, "Total");
        assert_eq!(ends(&window), ["blue", "red"]);
        window.get_by_label("+2");
    }

    /// A file with more indicators than the open pull-down shows at once,
    /// the last of which falls where the total rises and has a name far
    /// longer than the panel is wide: the pull-down scrolls to it. Closed,
    /// the pull-down takes the room it takes for a short file, before that
    /// name is chosen and after, and so does the map.
    #[test]
    fn the_last_of_many_indicators_can_be_scrolled_to_and_chosen() {
        let same: &[f64] = &[0.25, 0.25, 0.25];
        let many: Vec<(String, String)> = (0..40)
            .map(|at| (format!("X{at:02}"), format!("Indicator {at}")))
            .collect();
        let long = "A name that goes on ".repeat(6);
        let mut file: Vec<(&str, &str, &[f64])> = many
            .iter()
            .map(|(code, name)| (code.as_str(), name.as_str(), same))
            .collect();
        file.push(("Y00", &long, &[3.0, 0.0, -3.0]));
        file.push(("QOL", "Total", &[-2.0, 0.5, 1.0]));

        let short = window();
        let room = |window: &Harness<'_, Option<Window>>| {
            let (map, _) = meshes_drawn(window);
            (window.get_by_role(Role::ComboBox).rect(), map)
        };
        let mut window = window_on(dataset_of(&MESHES, &file));
        assert_eq!(room(&window), room(&short));
        assert_eq!(ends(&window), ["red", "blue"]);

        choose(&mut window, &long);
        assert_eq!(ends(&window), ["blue", "red"]);
        assert_eq!(room(&window), room(&short));
    }

    /// Presses and lets go on the slider, `along` of the way from its left
    /// end to its right.
    fn slide_to(window: &mut Harness<'_, Option<Window>>, along: f32) {
        let slider = window.get_by_role(Role::Slider).rect();
        let at = slider.left_center() + eframe::egui::vec2(slider.width() * along, 0.0);
        window.drag_at(at);
        window.run();
        window.drop_at(at);
        window.run();
    }

    /// How much of the base map the first mesh's square covers, and the
    /// legend's bar past its ground, each of 255.
    fn covered(window: &Harness<'_, Option<Window>>) -> [u8; 2] {
        let (_, corners) = meshes_drawn(window);
        let bar = window
            .output()
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Mesh(triangles) if triangles.vertices.len() != 4 * MESHES.len() => {
                    Some(triangles.vertices[4].color)
                }
                _ => None,
            })
            .expect("the window draws the legend's bar");
        [corners[0].color.a(), bar.a()]
    }

    /// The slider is under its label. The meshes open letting the base map
    /// through; slid to the right end they hide it, to the left end they
    /// leave it as it is, and in between they cover more of it the further
    /// right. The legend's bar follows, and the colours keep their ends.
    #[test]
    fn the_slider_sets_how_much_of_the_base_map_the_meshes_cover() {
        let mut window = window();
        let label = window.get_by_label(OPACITY_LABEL).rect();
        let slider = window.get_by_role(Role::Slider).rect();
        assert!(label.bottom() <= slider.top(), "{label:?} {slider:?}");
        assert_eq!(covered(&window), [OPENING_OPACITY; 2]);

        slide_to(&mut window, 1.0);
        assert_eq!(covered(&window), [255; 2]);
        assert_eq!(ends(&window), ["blue", "red"]);

        slide_to(&mut window, 0.0);
        assert_eq!(covered(&window), [0; 2]);

        slide_to(&mut window, 0.25);
        let [faint, _] = covered(&window);
        slide_to(&mut window, 0.75);
        let [strong, bar] = covered(&window);
        assert!(
            0 < faint && faint < strong && strong < 255,
            "{faint} {strong}"
        );
        assert_eq!(bar, strong);
    }

    /// Another indicator is painted as the slider was left, not as the tool
    /// opens.
    #[test]
    fn what_the_slider_set_stays_when_another_indicator_is_chosen() {
        let mut window = window();
        slide_to(&mut window, 1.0);
        choose(&mut window, "Floods");
        assert_eq!(covered(&window), [255; 2]);
        assert_eq!(ends(&window), ["red", "blue"]);
    }

    /// Moves the pointer to the middle of the square drawn for the mesh at
    /// `at` in [`MESHES`].
    fn point_at(window: &mut Harness<'_, Option<Window>>, at: usize) {
        let (_, corners) = meshes_drawn(window);
        let places: Vec<_> = corners[4 * at..][..4]
            .iter()
            .map(|corner| corner.pos)
            .collect();
        window.hover_at(Rect::from_points(&places).center());
        window.run();
    }

    /// The last mesh, whose total is -1 and whose value of the indicator is
    /// 3: the legend has neither as a mark, so each is found as the readout
    /// alone writes it.
    #[test]
    fn pointing_at_a_mesh_reads_out_its_code_its_municipality_and_the_value_shown() {
        let mut window = window();
        for label in [CODE_LABEL, MUNICIPALITY_LABEL, VALUE_LABEL] {
            window.get_by_label(label);
        }
        assert!(window.query_by_label(MESHES[2]).is_none());

        point_at(&mut window, 2);
        window.get_by_label(MESHES[2]);
        window.get_by_label(&city_of(MESHES[2]));
        window.get_by_label("-1");

        point_at(&mut window, 0);
        window.get_by_label(MESHES[0]);
        window.get_by_label(&city_of(MESHES[0]));
        assert!(window.query_by_label(MESHES[2]).is_none());
        assert!(window.query_by_label("-1").is_none());
    }

    /// The pull-down is chosen from with the pointer, which leaves the mesh
    /// to do so and comes back to it.
    #[test]
    fn the_value_read_out_is_that_of_what_the_colours_show() {
        let mut window = window();
        choose(&mut window, "Floods");
        point_at(&mut window, 2);
        window.get_by_label(MESHES[2]);
        window.get_by_label("3");
        assert!(window.query_by_label("-1").is_none());
    }

    /// The meshes are in the middle of the map, and its corner is far from
    /// them. The readout takes the same room with a mesh and without.
    #[test]
    fn pointing_away_from_the_meshes_reads_out_nothing() {
        let mut window = window();
        let room = |window: &Harness<'_, Option<Window>>| {
            [CODE_LABEL, MUNICIPALITY_LABEL, VALUE_LABEL]
                .map(|label| window.get_by_label(label).rect())
        };
        let without = room(&window);
        point_at(&mut window, 2);
        window.get_by_label(MESHES[2]);
        assert_eq!(room(&window), without);

        let (map, _) = meshes_drawn(&window);
        window.hover_at(map.left_top() + eframe::egui::vec2(5.0, 5.0));
        window.run();
        assert!(window.query_by_label(MESHES[2]).is_none());
        assert!(window.query_by_label(&city_of(MESHES[2])).is_none());
        assert!(window.query_by_label("-1").is_none());
        assert_eq!(room(&window), without);
    }

    /// The map dragged until the meshes are under the panel beside it, and
    /// the pointer put where the last of them would be drawn: it is on the
    /// panel and not on the map, so it is on no mesh, during the drag, which
    /// began on that mesh, and after it.
    #[test]
    fn pointing_at_the_panel_over_a_mesh_reads_out_nothing() {
        let mut window = window();
        let (map, corners) = meshes_drawn(&window);
        let places: Vec<_> = corners[8..].iter().map(|corner| corner.pos).collect();
        let from = Rect::from_points(&places).center();
        let to = eframe::egui::pos2(map.left() / 2.0, from.y);
        // A drag has the map drawn again and again, so frames are counted
        // out. The pointer rests before it lets go, so that the map does too.
        window.hover_at(from);
        window.run();
        window.get_by_label(MESHES[2]);
        window.drag_at(from);
        window.run_steps(2);
        window.hover_at(to);
        window.run_steps(10);
        assert!(window.query_by_label(MESHES[2]).is_none());
        window.drop_at(to);
        window.run_steps(10);

        let (map, corners) = meshes_drawn(&window);
        let places: Vec<_> = corners[8..].iter().map(|corner| corner.pos).collect();
        let square = Rect::from_points(&places);
        assert!(square.right() < map.left(), "{square:?} beside {map:?}");
        window.hover_at(square.center());
        window.run_steps(3);
        assert_eq!(meshes_drawn(&window).1, corners, "the map came to rest");
        assert!(window.query_by_label(MESHES[2]).is_none());
        assert!(window.query_by_label("-1").is_none());
    }

    /// A file with many indicators whose names are far longer than the panel
    /// is wide: open, the pull-down reaches over the map and covers the last
    /// mesh. The pointer where that mesh is drawn is on a choice and not on
    /// the map, so it is on no mesh.
    #[test]
    fn pointing_at_a_choice_over_a_mesh_reads_out_nothing() {
        let same: &[f64] = &[0.25, 0.25, 0.25];
        let long = " with a name that goes on".repeat(5);
        let many: Vec<(String, String)> = (0..40)
            .map(|at| (format!("X{at:02}"), format!("Indicator {at}{long}")))
            .collect();
        let mut file: Vec<(&str, &str, &[f64])> = many
            .iter()
            .map(|(code, name)| (code.as_str(), name.as_str(), same))
            .collect();
        file.push(("QOL", "Total", &[2.0, 0.5, -1.0]));
        let mut window = window_on(dataset_of(&MESHES, &file));
        let (_, corners) = meshes_drawn(&window);
        let places: Vec<_> = corners[8..].iter().map(|corner| corner.pos).collect();
        let mesh = Rect::from_points(&places).center();

        open_the_choices(&mut window);
        let covered = many
            .iter()
            .flat_map(|(_, name)| window.query_all_by_label(name))
            .any(|choice| choice.rect().contains(mesh));
        assert!(covered, "no choice lies over the mesh at {mesh:?}");
        window.hover_at(mesh);
        window.run();
        assert!(window.query_by_label(MESHES[2]).is_none());
        assert!(window.query_by_label("-1").is_none());
    }

    /// A municipality's name and a value both far longer than the panel is
    /// wide: the panel stays as wide, so the map stays where it is and the
    /// pointer stays on the mesh, which is read out in full.
    #[test]
    fn a_long_name_and_a_long_value_leave_the_map_where_it_is() {
        let city = ["A municipality whose name goes on"; 3].join(" and ");
        // As many figures as a value has at most.
        let value = (1.0_f64 / 3.0).to_string();
        let mut file = String::from(
            "KeyCode,PrefectureCode,CityCode,Prefecture,City,IndicatorCode,Indicator,Value\n",
        );
        for code in MESHES {
            file.push_str(&format!(
                "{code},00,00000,a prefecture,{city},QOL,Total,{value}\n"
            ));
        }
        let dataset = Dataset::read(file.as_bytes()).expect("a file made to be read");
        let mut window = window_on(dataset);
        let (map, _) = meshes_drawn(&window);

        point_at(&mut window, 2);
        assert_eq!(meshes_drawn(&window).0, map);
        for label in [CODE_LABEL, MUNICIPALITY_LABEL, VALUE_LABEL] {
            let label = window.get_by_label(label).rect();
            let said = window.get_by_label(MESHES[2]).rect();
            assert!(label.right() < said.left(), "{label:?} {said:?}");
        }
        window.get_by_label(&city);
        window.get_by_label(&value);
    }

    /// A reach too large and one too small for the legend to write in the
    /// room a mark has: the panel stays as wide, so the map is where a file of
    /// ordinary values has it.
    #[test]
    fn a_mark_too_long_for_the_legend_leaves_the_map_where_it_is() {
        let (map, _) = meshes_drawn(&window());
        for reach in [1e-300, 1e300] {
            let values: &[f64] = &[-reach, 0.0, reach];
            let window = window_on(dataset_of(&MESHES, &[("QOL", "Total", values)]));
            assert_eq!(meshes_drawn(&window).0, map, "a reach of {reach:e}");
        }
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
        let lowest = window.get_by_label("-2").rect();
        let middle = window.get_by_label("0").rect();
        let highest = window.get_by_label("+2").rect();

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
            OPACITY_LABEL,
            CODE_LABEL,
            MUNICIPALITY_LABEL,
            VALUE_LABEL,
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
