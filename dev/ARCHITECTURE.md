# Architecture

## Bird's-eye view

QOL Reweight is a native desktop application in Rust [0002].
Started with the path of one CSV file of Urban QOL data, it reads the file into memory, opens a window, and draws each of the file's 500 m meshes as a filled square over a base map of raster tiles.
The colour of a square is the value the file publishes for that mesh, the total or one indicator, on a scale fitted to the values being shown.
The file stays on its user's disk and is the tool's only source of data [0001].

## Code map

The tool is one binary crate.
Each module is built around one type, which the others use through its methods.

### [`src/main.rs`](../src/main.rs)

Reads the command line, opens the file, finds the font, and opens the window.
Where any of the three fails it prints why and exits without a window.

### [`src/mesh.rs`](../src/mesh.rs)

`HalfMesh`, a 500 m mesh as its row and column among all such meshes: read from a nine-digit code, found from a latitude and a longitude, and giving its four edges in degrees.
`Extent` is the area some meshes cover.

### [`src/dataset/`](../src/dataset/mod.rs)

`Dataset`, a file in memory: its meshes, each with its code, its square, and its municipality, and its series, the total first and then each indicator, each with one value for every mesh.
[`file.rs`](../src/dataset/file.rs) reads a CSV file into one, and refuses a file that is not laid out as the platform's are, naming the line.

### [`src/scale.rs`](../src/scale.rs) and [`src/paint.rs`](../src/paint.rs)

`Scale` fits a set of values: its reach is the size that most of their absolute values stay within, and it gives a colour between two hues, neutral at zero.
`Paint` is a scale with an opacity, and is the one place a value becomes the colour drawn on screen.

### [`src/layer.rs`](../src/layer.rs), [`src/legend.rs`](../src/legend.rs), and [`src/readout.rs`](../src/readout.rs)

What the window shows of the data.
`MeshLayer` holds each mesh's square with its colour and turns them into one shape for the map's projection.
`Legend` and `Readout` are egui widgets: the bar of colours with the values its ends stand for, and the code, municipality, and value of the mesh pointed at.

### [`src/basemap.rs`](../src/basemap.rs) and [`src/view.rs`](../src/view.rs)

`PaleMap` is the base map's tile source, GSI's pale map, with the statement of its source.
`View` is where the map opens: centred on the file's meshes, at a fixed zoom level.

### [`src/font.rs`](../src/font.rs) and [`src/asset.rs`](../src/asset.rs)

`JapaneseFont` is the font for Japanese text, found among the system's fonts by its family's name.
`asset::text!` takes in a text under [`assets/`](../assets/), where the Japanese text the screen shows is kept.

### [`src/window.rs`](../src/window.rs)

`Window` holds the dataset, what is chosen on screen, and the map's state, and lays out each frame: the statements of the sources at the bottom, the legend, the pull-down, and the slider in a panel on the left, the map with the meshes in the rest, and the readout beside the pointer while it is on a mesh.
Its tests drive it without showing it.

### [`tests/starts.rs`](../tests/starts.rs)

Starts the built tool as a user would, for the starts that end without a window.

## Invariants

- The tool sends the platform no request. The base map's tiles are its only use of the network, and the meshes are drawn without it [0001].
- The tool reads the file and writes nothing: no file, no cache of tiles, and no saved state of the window.
- No frame shows the map without the statements of the data's source and of the base map's. Without a font that draws them, no window opens.
- The scale's reach is worked out from the values being shown. No threshold taken from the data is written in the code.
- A square's colour comes from `Paint` alone, on the map and in the legend.
- A mesh's edges are computed from whole counts of rows and columns, so two meshes that share an edge get the same coordinate for it and are drawn with no gap.
- Source files hold no Japanese text, and tests make their input in code, as [`CLAUDE.md`](../CLAUDE.md#code-conventions) says. Nothing that comes from the data is in the repository.

## Boundaries and outside-visible behavior

- **The file**: one of the platform's CSV files crosses into the tool, by a path on the command line. Its layout is the one the [phase 01 plan](plan/phase-01-local-map.md) records as A001, and its mesh codes are taken to be half grid square codes on the base map's geodetic system, which that plan's A002 and A003 checked against public data for the files of three regions. No test holds this for a file of another region.
- **The tile server**: requests for GSI's pale map tiles go out, and images come back. Nothing from the file goes with them.
- **The system's fonts**: Hiragino Sans is read from where the system keeps it.
- Over the base map, the meshes lie on the land it draws, and it names places in Japanese under them. No test holds this.
- Panning and zooming stay smooth with a metropolitan area's file open. No test holds this either.
- Started without a path, or with a file it cannot read, the tool prints which file it expects and where to download it, and exits with a failure.
- The [README](../README.md) says how to get a file and start the tool, that only macOS is checked, and what happens without the font.

## Cross-cutting concerns

- **Errors**: each module that can fail has its own error type, derived with `thiserror`, and `main` prints it. Nothing is recovered from: a file that cannot be read in full is not shown in part.
- **Testing**: unit tests sit in each module's file, on synthetic data that [`src/test_support.rs`](../src/test_support.rs) makes. The tests that drive the window need the font, so they run on macOS only; CI runs on Linux, where the start-up test for a system without the font runs instead. One test, in `src/window.rs`, reads real files from a folder outside the repository and checks the tool against a second reading of each; it runs only when asked for, as the [README](../README.md#development) says.
- **Text on screen**: every Japanese text is a file under `assets/`, and [`GLOSSARY.md`](GLOSSARY.md) pairs each English name in the code with its Japanese term.

[0001]: decisions/0001-csv-kept-on-disk-as-data-source.md
[0002]: decisions/0002-native-rust-app-on-egui-and-walkers.md
