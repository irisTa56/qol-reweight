# Phase 01: Published values on a local map

- Status: In progress
- Roadmap: [Published values on a local map](../ROADMAP.md#phases)

## Goal

On the maintainer's machine, the tool opens one prefecture's CSV file and shows its published values on a map of 500 m meshes, the total or one indicator at a time.
The picture agrees with the one the platform shows for the same file, which settles the path from the file to the map.

## Requirements & Constraints

- **R001**: Started with the path of a prefecture's CSV file, the tool opens a window with a map on which every mesh of the file is a filled square, at the place its mesh code names, coloured by a published value.
- **R002**: The user chooses what the colour shows: the published total, or any one indicator the file carries. The choices are read from the file.
- **R003**: The colours run between two hues through a neutral colour at zero, and a legend on screen says which colour stands for which value.
  - The two ends stand for plus and minus one value, a high percentile of the absolute values being shown and not their largest, so that a few extreme meshes do not leave the rest near the neutral colour.
  - That value is worked out when what is shown is chosen, and is written nowhere in the code.
- **R004**: Pointing at a mesh shows its mesh code, the name of its municipality, and the value being shown.
- **R005**: The meshes lie over a base map that names places in Japanese, and the map can be panned and zoomed.
  - It does not zoom out past level 9, below which the base map's tiles ask for a further credit (A005).
- **R006**: Whenever the map is shown, the screen states the data's source, as `都市QOLデータ2020（大日本ダイヤコンサルタント㈱作成）`, and the base map's source with its link.
  - Where the tool cannot load a font that renders those statements, it says so and shows no map.
- **R007**: Started without a path, or with a file it cannot read as Urban QOL data, the tool says which file it expects and where its user downloads it, and shows no map.
  - The README says the same, and how to start the tool.
- **R008** (constraint): The tool sends the platform no request ([0001]). The base map's tiles are its only use of the network, and the meshes are drawn without it.
- **R009** (constraint): The tool reads the file and writes nothing that comes from it.
- **R010** (constraint): Nothing that comes from the data is committed, as [`CLAUDE.md`](../../CLAUDE.md#keeping-the-data-out-of-the-repository) spells out, so the tests run on synthetic data and the phase's evidence holds no picture and no value.
- **R011** (constraint): The tool is checked on macOS only, and reads the font for Japanese text from there. On Windows and Linux it is not checked, and where it finds no such font it shows no map, as R006 says; the README says so.

### Out of scope

- Multipliers, several files at once, and a total that is not the sum of its indicators: each is a later phase of the roadmap. A file whose total is not that sum is shown here as published.
- A metropolitan area's file: none has been read, so this phase does not claim to open one, and the [roadmap](../ROADMAP.md#phases) says when its phase is decided.
- Running in a browser, a base map of vector tiles, and a base map that works without the network.
- A colour range the user sets.

## Assumptions & Risks

- **A001**: A prefecture's file has the layout the two files read have. Source: the files of two prefectures, checked 2026-10-05.
  - It is UTF-8, with or without a byte order mark, and has one header row.
  - It has one row for each mesh and indicator, and the total is the row whose `IndicatorCode` is `QOL`.
  - Its columns are:
    - `KeyCode`
    - `PrefectureCode`
    - `CityCode`
    - `Prefecture`
    - `City`
    - `IndicatorCode`
    - `Indicator`
    - `Value`
  - Risk: another prefecture's file is laid out differently, noticed by the tool refusing a file downloaded from the platform.
- **A002**: `KeyCode` is a half grid square code: nine digits whose ninth is 1 to 4 for the south-west, south-east, north-west, and north-east quarter of a 1 km mesh, each 15″ of latitude by 22.5″ of longitude. Source: [地域メッシュ統計の特質・沿革](https://www.stat.go.jp/data/mesh/pdf/gaiyo1.pdf), pp. 8–12 and 19, checked 2026-10-05; every code in the two files has nine digits and ends in 1 to 4.
  - Risk: squares land in the wrong place, noticed by a pattern that is mirrored or scrambled within each 1 km against the platform's map.
- **A003**: The mesh codes are on the world geodetic system the base map uses, as the [2020 census mesh statistics](https://www.stat.go.jp/data/mesh/index.html) are. Unverified: nothing read says which datum the data uses.
  - Risk: every mesh is displaced, noticed by one steady offset against the coastline and against the platform's map.
- **A004**: The platform draws each file's meshes on a map, where its user chooses the total or one indicator, which is the picture the phase compares against. Source: [the platform's search result for the catalogue](https://data-platform.mlit.go.jp/#/searchlink/df633780-e1bd-436d-b6f6-13885a70c254), on whose map the maintainer saw Aichi's meshes coloured, and the colours change on choosing the total or an indicator in the legend, checked 2026-10-05.
  - Risk: the platform's map changes or stops showing a file, noticed on opening it for the two prefectures.
- **A005**: An application may load GSI tiles as they are needed without applying, on stating the source as 「国土地理院」 or 「地理院タイル」 with a link to the tile list. Source: [地理院タイル一覧](https://maps.gsi.go.jp/development/ichiran.html), checked 2026-10-05. The page says nothing of how many requests are allowed.
  - The pale map's tiles at zoom level 8 and below ask for a credit to their shoreline data as well, in the notes of their own entries on that page.
  - Risk: tiles are refused, noticed by a base map that stays blank while the network is up.
- **A006**: A file's meshes can all be drawn again on every frame. Source: the measurement in [0002], on synthetic data.
  - Risk: a real file is slower, noticed by panning that stutters with a prefecture's file open.
  - Risk: the map stops being drawn again, as one benchmark run did for a cause that was not found ([0002], under its risks), noticed by a map that stops following the pointer while its window is in front.

## Decisions

- [0002. Build the tool as a native Rust application on egui and walkers](../decisions/0002-native-rust-app-on-egui-and-walkers.md)
- The file's path is an argument on the command line rather than a file dialog or a fixed directory in the working tree, because it adds no dependency and leaves the file outside the working tree, where no commit can pick it up.
- The base map is GSI's pale raster tiles (淡色地図) rather than OpenStreetMap's standard tiles, OpenFreeMap's vector tiles, or none, because it is the one whose picture and terms were both checked: a demonstration drew it with Japanese labels (run 2026-10-05), and A005 holds its terms.
  - The map stops zooming out at level 9 rather than adding the credit the tiles below it ask for, because a prefecture fits in the window at level 9: a window 1,280 pixels wide spans about 3.5° of longitude there.
  - OpenStreetMap's [tile usage policy](https://operations.osmfoundation.org/policies/tiles/) allows blocking heavy use without notice, and whether its labels or OpenFreeMap's come out in Japanese was not checked.
- The colours are a continuous scale between two hues, centred on zero, with its ends at plus and minus a high percentile of the absolute values shown, rather than their largest, classes of equal count, or a range the user sets, because the published values are differences from a mean, so their sign carries meaning ([the data's introduction](https://data-platform.mlit.go.jp/#/Page?id=dataintro01), checked 2026-10-04), and a threshold taken from the data cannot be written in the code.
- The phase is checked against the files of Aichi and Tokyo rather than Aichi's alone, because the two differ in how many indicators they carry and in the byte order mark, and both are already on the maintainer's machine. A metropolitan area's file would take another download.
- Pointing at a mesh shows its numbers rather than leaving colour and legend alone, because a mesh can then be checked by its value as well as by its colour.
- The tool is checked on macOS only rather than made to show Japanese text everywhere, because it is a personal tool on a macOS machine, and a font shipped with it would be a file to add and a licence to check.
  - Without the font it shows no map rather than a map whose source statements cannot be read, because the roadmap makes stating the data's source a condition of showing the data.

## Dependencies

- **Rust toolchain**: builds and tests the tool; declared in `mise.toml` beside the tools already there.
- **`eframe`, `egui`, and `walkers`**: the window and controls, and the map widget with its tile fetching ([0002]).
- **A CSV reader**: parses the file; the crate is chosen while building.
- **Rust checks in `mise.toml` and CI**: formatting, lints, and tests join the tasks that gate a commit and a pull request.

## Done when

- **With Aichi's file, the map of the total shows the pattern the platform's map shows: high and low places fall in the same places, and the meshes end at the same coastline and prefectural border. Panning and zooming stay smooth.** — verifies R001, R005, A002, A003, A004, A006.
  - Check: the maintainer looks at the tool and the platform's map side by side, for the total and for two indicators. The evidence is the statement that they agree, with no picture.
- **With Tokyo's file, the same holds, and the list of choices is that file's indicators and not Aichi's.** — verifies R001, R002, A001.
  - Check: as above, by the maintainer.
- **Mesh codes turn into the squares the standard defines, files that differ in indicators and in the byte order mark load, and zero gets the neutral colour with the two ends at plus and minus a high percentile of the absolute values shown.** — verifies R001, R002, R003, A001, A002.
  - Check: automated tests on synthetic data, passing in CI.
- **Choosing another indicator recolours the map and changes the legend, and pointing at a mesh shows its code, its municipality, and its value.** — verifies R002, R003, R004.
  - Check: by hand with a synthetic file whose values are known from how it was made, and again with Aichi's file.
- **The data's source and the base map's source, with its link, are on screen whenever the map is, and the map does not zoom out past level 9.** — verifies R005, R006.
  - Check: read off the window with a synthetic file open, and zooming out tried by hand.
- **Started with no path, with a path that does not exist, and with a file that is not Urban QOL data, the tool prints which file it expects and where it comes from, and exits without a map. Started where the font cannot be loaded, it says so and exits without a map. The README says how to get the file and start the tool, that only macOS is checked, and what happens without the font.** — verifies R006, R007, R011.
  - Check: an automated test for the four starts, and the README read.
- **With the network off, the meshes are drawn over a blank base map, and the tool makes no request to the platform.** — verifies R008.
  - Check: the tool run offline with a synthetic file, and each place the code names the platform's host read to be text the tool shows, not an address it requests.
- **The tool leaves no file behind that holds anything from the CSV, and the repository holds no data: test inputs are made by code, and no commit of the phase carries a picture or a value from a real file.** — verifies R009, R010.
  - Check: the code read for what it writes, and the check before each push that `CLAUDE.md` asks for.
- A005 has no item of its own: its source is the terms page, and its warning sign would appear in the first item.

## Open questions

None.

[0001]: ../decisions/0001-csv-kept-on-disk-as-data-source.md
[0002]: ../decisions/0002-native-rust-app-on-egui-and-walkers.md
