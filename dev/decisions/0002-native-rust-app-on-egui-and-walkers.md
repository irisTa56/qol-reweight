# 0002. Build the tool as a native Rust application on egui and walkers

- Status: Proposed
- Date: 2026-10-05

## Context

The tool draws every 500 m mesh of a file as a coloured square over a base map, and the [roadmap](../ROADMAP.md#scope) has that map follow the multipliers as the user moves them.
What the tool is built on has to be chosen before the first map exists.

### What the maintainer said

Three things the maintainer said bear on the choice.

- Of everything the design covers, the maintainer weighs most how the map follows a multiplier while it is being moved (the request this project started from, 2026-10-04).
- The maintainer will not write JavaScript or TypeScript by hand, and accepts JavaScript that a tool generates or a library ships (said while this record was prepared, 2026-10-05).
- Between a Rust application and Python, the maintainer prefers Rust, and calls the choice of Rust a preference in the end (said while this record was prepared, 2026-10-05).

### The size of the job

The files set the size of the job.

- A prefecture's file holds on the order of ten thousand meshes, and the largest file is several times that, judging by the files' sizes ([the platform's catalogue](https://data-platform.mlit.go.jp/#/searchlink/df633780-e1bd-436d-b6f6-13885a70c254), checked 2026-10-04).
- A file carries up to 26 indicators (the two files read, checked 2026-10-05).

### Redraw speed, measured

Redraw speed was measured on synthetic data: squares laid out as a grid, 26 made-up values each, coloured by their weighted sum, with one multiplier changed per update, on an Apple M3 (run 2026-10-05).

- **[egui](https://github.com/emilk/egui) 0.36 with [walkers](https://github.com/podusowski/walkers) 0.60.0, as a native application**, rebuilding every square on every frame: the tool's own work in a frame took a median of 2.3 ms for 12,000 squares, 4.3 ms for 45,000, and 7.1 ms for 120,000, and frames stayed 16.7 ms apart, the display's refresh interval, over 600 frames at each size.
- **[lonboard](https://developmentseed.org/lonboard/latest/) 0.17.0 inside [marimo](https://marimo.io/) 0.25.1**, where each slider event goes to a Python process that sends new colours to the browser: Python took about 4 ms for 12,000 squares and 7 ms for 45,000, and served about 46 events a second without falling behind.
  - From one event to a changed picture took a median of at most 57 ms for 45,000 squares; the measurement could not resolve less than about 40 ms, so how many frames that is stayed unknown.
- **[deck.gl](https://deck.gl/) 9.4.0 in Chrome 154**, alone or over [MapLibre GL JS](https://maplibre.org/maplibre-gl-js/docs/) 6.12.0: the change was drawn on the next frame up to 45,000 squares.
- **MapLibre GL JS 6.12.0 alone**, updating each square's feature state: the next frame for 12,000 squares, and a median of 48 ms for 45,000.
- **walkers' own layer for GeoJSON polygons** (`walkers_extras` 0.60.0), in place of the tool's own drawing: drawing the same squares again took 47.7 ms a frame for 12,000 and 182.4 ms for 45,000, with no value changed (run 2026-10-06).
- **[galileo](https://github.com/galileo-map/galileo) 0.2.1 with its egui widget**, recolouring a feature layer through its symbol: 11.8 ms a frame for 12,000 squares, 44.3 ms for 45,000, and 127.9 ms for 120,000, so frames came 44.6 ms apart at 45,000 (run 2026-10-06).
  - Its hit test returned the square under a point, its raster tile layer drew GSI tiles with an attribution it shows itself, and Japanese text showed with the same font added.

### What exists in Rust

What exists in Rust was taken from crates.io's search, [lib.rs's geo category](https://lib.rs/science/geo), [awesome-rust](https://github.com/rust-unofficial/awesome-rust), and [awesome-georust](https://github.com/pka/awesome-georust) (checked 2026-10-05).

- walkers is a map widget for egui under the MIT licence, released as 0.60.0 on 2026-09-22 ([docs.rs](https://docs.rs/crate/walkers/0.60.0)).
  - It draws raster tiles from a tile server one describes in a few lines, and hands a plugin a projection to draw shapes of one's own on the map (its `TileSource` and `Plugin` traits).
  - The map view of [Rerun](https://github.com/rerun-io/rerun) is built on it ([`re_view_map` 0.38.1](https://docs.rs/crate/re_view_map/0.38.1) depends on `walkers ^0.58.0`).
- A demonstration on synthetic data drew GSI raster tiles with their Japanese labels, 12,000 squares over them, and 26 sliders labelled in Japanese beside the map, in about 130 lines (run 2026-10-05).
  - egui's bundled fonts hold no Japanese glyphs ([`epaint_default_fonts` 0.36.2](https://docs.rs/crate/epaint_default_fonts/0.36.2/source/fonts/) ships Hack, Ubuntu Light, Noto Emoji, and an icon font), so the demonstration read a font from macOS.
- No Rust bindings for deck.gl turned up on crates.io or GitHub (searched 2026-10-05).
- Rerun's map view draws points and line strings only ([its reference](https://github.com/rerun-io/rerun/blob/0.38.1/docs/content/reference/types/views/map_view.md)); filled polygons are [an open request](https://github.com/rerun-io/rerun/issues/8066) from 2024-11-11.
  - Controls of one's own go in an eframe application wrapped around its viewer ([its example of extending the viewer](https://github.com/rerun-io/rerun/tree/0.38.1/examples/rust/extend_viewer_ui)).
- galileo's egui widget, at its latest release 0.2.1, depends on egui 0.31 (crates.io).
- [maplibre-rs](https://github.com/maplibre/maplibre-rs) lists text rendering as missing (its README).

### What exists in Python

What exists in Python was taken from [pyviz.org's list of tools](https://pyviz.org/tools.html) and [anywidget's community page](https://anywidget.dev/en/community/), and read from each tool's documentation or source (checked 2026-10-05).

- Streamlit sends a slider's value when it is released ([`Slider.tsx` at 1.65.0](https://github.com/streamlit/streamlit/blob/1.65.0/frontend/lib/src/components/widgets/Slider/Slider.tsx), where only `onChangeEnd` sets the value the server sees).
- pydeck 0.9 cannot update a map it has shown ([`deck.py` at v9.4.0](https://github.com/visgl/deck.gl/blob/v9.4.0/bindings/pydeck/pydeck/bindings/deck.py), whose `update` raises `NotImplementedError`).
- ipyleaflet sends the whole GeoJSON again when a style changes ([`leaflet.py` at 0.20.0](https://github.com/jupyter-widgets/ipyleaflet/blob/0.20.0/python/ipyleaflet/ipyleaflet/leaflet.py), where `_update_data` observes `style` and copies `data`).
- What would compute in the browser is JavaScript written by hand: Bokeh's [`CustomJS`](https://docs.bokeh.org/en/latest/docs/user_guide/interaction/js_callbacks.html), Panel's [`jslink`](https://github.com/holoviz/panel/blob/v1.9.4/doc/how_to/links/jslinks.md), and Dash's [clientside callbacks](https://dash.plotly.com/clientside-callbacks).

## Decision

The tool is a native desktop application written in Rust.
egui draws its window and controls, walkers draws the map and fetches the base map's tiles, and the tool's own code draws the meshes on that map.

## Rejected alternatives

- **TypeScript with deck.gl over MapLibre GL JS, in the browser**: it drew a change on the next frame, and it is JavaScript written by hand, which the maintainer ruled out.
- **Python with lonboard inside marimo, served by a Python process**: it kept up with the slider, but each change crosses to a Python process and back, and the measurement could not show that the picture changes within a frame or two, while the Rust application changes it on the next frame.
  - What the Rust application has to draw for itself, which was the reason to prefer this alternative, came to about 130 lines in the demonstration.
  - Between the two, the maintainer's preference is for Rust, as the context says.
  - marimo can also run a notebook's Python in the browser, with no Python process ([its guide to WebAssembly notebooks](https://docs.marimo.io/guides/wasm/), checked 2026-10-05). That was not tried, so nothing here says how lonboard follows a slider there.
- **Streamlit, pydeck, or ipyleaflet in place of marimo or lonboard**: Streamlit sends a slider's value too late to follow a drag, pydeck cannot update what it has drawn, and ipyleaflet sends the whole geometry on every change.
  - Other Python hosts and map libraries were not measured, and one that reacts to a slider in a Python process shares the crossing the alternative above is turned down for.
- **Computing in the browser from Bokeh, Panel, or Dash**: the hooks for it need JavaScript written by hand.
- **Rust compiled to WebAssembly, driving deck.gl or MapLibre GL JS**: deck.gl has no bindings, so its interface would be declared by hand, and MapLibre GL JS alone took 48 ms to recolour 45,000 squares.
- **Rerun's viewer**: its map view cannot fill a polygon, and controls of one's own mean wrapping the viewer in an egui application, which is this decision with a larger dependency around it.
- **galileo**: it does the whole job, with hit testing and the attribution built in, but took 44.3 ms to recolour 45,000 squares where the tool's own drawing on walkers took 4.3 ms, and its egui widget is on an older egui.
- **walkers' own layer for GeoJSON polygons, in place of the tool's own drawing**: it took 47.7 ms a frame to draw 12,000 squares that had not changed.
- **maplibre-rs**: it draws no text, by its own README.

## Consequences

- **Dependencies added**: the Rust toolchain, `eframe` and `egui`, and `walkers`, with what they bring in, a GPU renderer and an HTTP client among it.
- **Risks**:
  - walkers is before 1.0 and changes its interface between releases; 0.60 began to require egui's wgpu renderer for vector tiles ([its changelog](https://github.com/podusowski/walkers/blob/main/CHANGELOG.md), checked 2026-10-05). It would show as a build that fails after an upgrade.
  - Drawing the meshes, finding the mesh under the pointer, and the legend are the tool's own code. walkers' layer for GeoJSON polygons and galileo's feature layer would have drawn the meshes, and each was too slow, as the context measures.
  - Japanese text needs a font from the operating system, so the tool shows it only where it knows where that font is.
  - One benchmark run stopped receiving frames after about 150, and three later runs of 600 frames each did not; the first run did not record whether its window was visible, so the cause is unknown. It would show as a map that stops redrawing while its window is in front.
  - The tool is a window on the desktop and not a page, so running it in a browser would be a second build target, which walkers supports and nothing here tried.
