# Phase 01: Published values on a local map

- Status: In progress
- Roadmap: [Published values on a local map](../ROADMAP.md#phases)

## Goal

On the maintainer's machine, the tool opens one of the platform's CSV files, a prefecture's or a metropolitan area's, and shows its published values on a map of 500 m meshes, the total or one indicator at a time.
The path from the file to the map is settled without the platform's map: the tool's reading of each real file is checked against a second reading, where a mesh lies rests on assumptions checked against public reference data, and the map as drawn is looked at over the base map.

## Requirements & Constraints

- **R001**: Started with the path of one of the platform's CSV files, the tool opens a window with a map on which every mesh of the file is a filled square, at the place its mesh code names, coloured by a published value.
- **R002**: The user chooses on screen what the colour shows, from a list: the published total, which is the file's `QOL` row, or any one indicator. The list is made from the indicators found in the opened file.
- **R003**: The colours run between two hues through a neutral colour at zero, and a legend on screen says which colour stands for which value.
  - The two ends stand for plus and minus one value, a high percentile of the absolute values being shown and not their largest, so that a few extreme meshes do not leave the rest near the neutral colour.
  - That value is worked out when what is shown is chosen, and is written nowhere in the code.
- **R004**: Pointing at a mesh shows its mesh code, the name of its municipality, and the value being shown.
- **R005**: The meshes lie over a base map that names places in Japanese, and the map can be panned and zoomed.
  - The meshes let the base map show through, and the user can change on screen how much.
- **R006**: Whenever the map is shown, the screen states the data's source, as `都市QOLデータ2020（大日本ダイヤコンサルタント㈱作成）`, and the base map's source with its link.
  - Beside them it states the credit to shoreline data that the base map's tiles at small scales ask for (A005), at every zoom level.
  - Where the tool cannot load a font that renders those statements, it says so and shows no map.
- **R007**: Started without a path, or with a file it cannot read as Urban QOL data, the tool says which file it expects and where its user downloads it, and shows no map.
  - The README says the same, and how to start the tool.
- **R008** (constraint): The tool sends the platform no request ([0001]). The base map's tiles are its only use of the network, and the meshes are drawn without it.
- **R009** (constraint): The tool reads the file and writes nothing that comes from it.
- **R010** (constraint): Nothing that comes from the data is committed, as [`CLAUDE.md`](../../CLAUDE.md#keeping-the-data-out-of-the-repository) spells out, so the tests run on synthetic data and the phase's evidence holds no picture and no value.
  - A test that needs a real file takes it from outside the repository, runs only when asked for, and holds no value: what it expects is read from the file.
- **R011** (constraint): The tool is checked on macOS only, and reads the font for Japanese text from there. On Windows and Linux it is not checked, and where it finds no such font it shows no map, as R006 says; the README says so.

### Out of scope

- Multipliers, several files at once, and a file whose `QOL` row is not what its indicator rows add up to: each is a later phase of the roadmap. Such a file's `QOL` row is shown here as published, like any other.
- Running in a browser, a base map of vector tiles, and a base map that works without the network.
- A colour range the user sets.
- Running the tests that drive the window in CI, which would take a macOS runner.

## Assumptions & Risks

- **A001**: Each of the platform's files has the layout the three files read have. Source: the files of two prefectures and of one metropolitan area, checked 2026-10-06.
  - It is UTF-8, with or without a byte order mark, and has one header row.
  - It has one row for each mesh and indicator, and the total is the row whose `IndicatorCode` is `QOL`.
    - Every row of a mesh names the same `City`, and every row of an `IndicatorCode` the same `Indicator`; none of `City`, `IndicatorCode`, and `Indicator` is blank, and no `Value` has a space before or after it.
  - A metropolitan area's file holds the meshes of several prefectures in that one layout.
  - Its columns are:
    - `KeyCode`
    - `PrefectureCode`
    - `CityCode`
    - `Prefecture`
    - `City`
    - `IndicatorCode`
    - `Indicator`
    - `Value`
  - Risk: another file is laid out differently, noticed by the tool refusing a file downloaded from the platform.
- **A002**: `KeyCode` is a half grid square code: nine digits whose ninth is 1 to 4 for the south-west, south-east, north-west, and north-east quarter of a 1 km mesh, each 15″ of latitude by 22.5″ of longitude. Source: [地域メッシュ統計の特質・沿革](https://www.stat.go.jp/data/mesh/pdf/gaiyo1.pdf), pp. 8–12 and 19, checked 2026-10-05; every code in the three files has nine digits and ends in 1 to 4.
  - e-Stat's boundary data agrees: for every mesh in its files of the first-level squares the three files cover, the square these rules give is the square the file holds, to within the rounding of the arithmetic, and every mesh of the three files is in them (a trial script, run 2026-10-10).
    - The data is on its [list of boundary files](https://www.e-stat.go.jp/gis/statmap-search?page=1&type=2&aggregateUnitForBoundary=H&coordsys=1&format=shape), which offers the 500 m meshes, its fourth-level meshes, as Shapefiles in latitude and longitude on the world geodetic system, one for each first-level square, dated 2010-12-08 (checked 2026-10-09).
    - Each record is one rectangle with a nine-digit `KEY_CODE`, as the definition document on that page says, and the files name their geodetic system as JGD2000. A file does not hold every mesh of its first-level square: some of those opened hold fewer.
    - Its content may be used under [e-Stat's terms](https://www.e-stat.go.jp/terms-of-use), which follow the government's standard terms of use 2.0 and are compatible with CC BY 4.0; the [terms of its GIS](https://www.e-stat.go.jp/gis-terms) refer to them for what the system provides (checked 2026-10-09).
  - The codes of the three files are those of the 2020 census's statistics on 500 m meshes, quarters included: nearly every mesh of each file has a row in the census's table of population, and many times as many lack one once the quarters of each 1 km mesh are renumbered, east for west, north for south, or both (a trial script, run 2026-10-10).
    - The table is e-Stat's table of population and households, `T001141`, of the 2020 census on its fourth-level meshes, in its JGD2011 edition, on its [list of statistics](https://www.e-stat.go.jp/gis/statmap-search?page=1&type=1&toukeiCode=00200521&toukeiYear=2020&aggregateUnit=H), dated 2025-10-09, one file for each first-level square (checked 2026-10-10). A file has a row with a nine-digit `KEY_CODE` for a part of its square's meshes, taken here to be those where the census counted someone.
    - A few meshes of each file have no row as they are. Why was not looked into.
    - e-Stat's terms, above, cover it.
  - Risk: the codes of a file follow other rules, so its squares land in the wrong place, noticed by meshes drawn off the land, which shows a gross error and not a wrong order of the quarters, and by running the two trials on that file.
- **A003**: The mesh codes are on the world geodetic system the base map uses, as the [2020 census mesh statistics](https://www.stat.go.jp/data/mesh/index.html) are. Source: the trial against the census's table under A002, in which many times as many meshes of each file lack a row once every mesh is moved by as far as the older Japanese datum lies from the world geodetic system, in either direction (run 2026-10-10).
  - Those statistics are compiled in two such systems, JGD2000 and JGD2011, and nothing read says which the data uses. The two differ by far less than a mesh: JGD2011 revised JGD2000 for the ground the 2011 earthquake moved, and the origin of latitude and longitude, in Tokyo, moved by about 27 cm ([GSI](https://www.gsi.go.jp/sokuchikijun/jgd2000-2011.html), checked 2026-10-09).
  - Risk: every mesh is displaced, as it would be were the codes on the older Japanese datum, which lies about 450 m from the world geodetic system near Tokyo ([GSI](https://www.gsi.go.jp/LAW/G2000-g2000-h3.htm), checked 2026-10-05), noticed by one steady offset of the meshes against the coast the base map draws.
  - Risk: a file of another region is on another system than these three, noticed by running the same trial on that file.
- **A004**: Withdrawn 2026-10-09, when the phase stopped comparing against the platform's map (see Decisions). It read: the platform draws each file's meshes on a map, where its user chooses the total or one indicator, which is the picture the phase compares against. Source: [the platform's search result for the catalogue](https://data-platform.mlit.go.jp/#/searchlink/df633780-e1bd-436d-b6f6-13885a70c254), on whose map the maintainer saw the meshes coloured, and the colours change on choosing the total or an indicator in the legend, for Aichi (checked 2026-10-05) and for Tokyo and the Chukyo metropolitan area (checked 2026-10-06).
- **A005**: An application may load GSI tiles as they are needed without applying, on stating the source as 「国土地理院」 or 「地理院タイル」 with a link to the tile list. Source: [地理院タイル一覧](https://maps.gsi.go.jp/development/ichiran.html), checked 2026-10-05. The page says nothing of how many requests are allowed.
  - The pale map's tiles at zoom level 8 and below ask for a credit to their shoreline data as well, in the notes of their own entries on that page.
  - Risk: tiles are refused, noticed by a base map that stays blank, at a zoom level it has tiles for, while the network is up.
- **A006**: A file's meshes can all be drawn again on every frame. Source: the measurement in [0002], on synthetic data.
  - Risk: a real file is slower, noticed by panning that stutters with a file open.
  - Risk: the map stops being drawn again, as two benchmark runs did for a cause that was not found ([0002], under its risks), noticed by a map that stops following the pointer while its window is in front.
- **A007**: A test can drive the tool's window, the map and the meshes included, without showing it. Source: a trial with [`egui_kittest`](https://docs.rs/egui_kittest/0.36.2) 0.36.2 on macOS, which found a label by its text, moved the pointer over a square, and rendered the window to an image (run 2026-10-06).
  - Risk: something the phase must check cannot be reached that way, noticed by a check below that ends up done by hand.

## Decisions

- [0002. Build the tool as a native Rust application on egui and walkers](../decisions/0002-native-rust-app-on-egui-and-walkers.md)
- The file's path is an argument on the command line rather than a file dialog or a fixed directory in the working tree, because it adds no dependency and leaves the file outside the working tree, where no commit can pick it up.
- The base map is GSI's pale raster tiles (淡色地図) rather than OpenStreetMap's standard tiles, OpenFreeMap's vector tiles, or none, because it is the one whose picture and terms were both checked: a demonstration drew it with Japanese labels (run 2026-10-05), and A005 holds its terms.
  - The screen states the credit the tiles at zoom level 8 and below ask for at every zoom level, rather than only while such tiles are shown or stopping the map at level 9, because the tool cannot tell which level's tiles the map widget has drawn, and whether a file fits in the window at level 9 depends on the prefecture and on the window.
  - OpenStreetMap's [tile usage policy](https://operations.osmfoundation.org/policies/tiles/) allows blocking heavy use without notice, and whether its labels or OpenFreeMap's come out in Japanese was not checked.
- The colours are a continuous scale between two hues, centred on zero, with its ends at plus and minus a high percentile of the absolute values shown, rather than their largest, classes of equal count, or a range the user sets, because the published values are differences from a mean, so their sign carries meaning ([the data's introduction](https://data-platform.mlit.go.jp/#/Page?id=dataintro01), checked 2026-10-04), and a threshold taken from the data cannot be written in the code.
- The phase is checked against the files of Aichi, Tokyo, and the Chukyo metropolitan area rather than Aichi's alone, because between them the three have both sets of indicators the catalogue lists, 16 and 26, a file with and a file without the byte order mark, and a file that holds several prefectures.
- Pointing at a mesh shows its numbers rather than leaving colour and legend alone, because a mesh can then be checked by its value as well as by its colour.
- The user sets how much of the base map shows through the meshes rather than the tool fixing it, because the maintainer finds a place hard to tell where the base map cannot be seen (said 2026-10-05).
- What can be checked by a test that drives the window is checked that way rather than by hand, because the test goes on guarding the behaviour after the phase closes.
  - Those tests run on macOS on the maintainer's machine rather than in CI, because CI runs on Linux, where the tool shows no map for want of the font (R011), and the maintainer starts with CI as it is (said 2026-10-06).
- The path from the file to the map is settled without the platform's map rather than by eye against it, because the eye had no way to the platform that can be repeated: it refused a browser driven by a program (HTTP 403, tried 2026-10-05, and again 2026-10-09 with two browsers), and no picture of real data can be kept to compare against (the maintainer, 2026-10-09).
  - The tool's reading of a real file is checked by a test against a second reading of that file, which runs on the maintainer's machine when asked for rather than in CI, because the repository holds no real file.
  - Where a mesh lies rests on A002 and A003, each checked once against public reference data rather than by a test kept with the code, because what was checked is the standard's rules and the geodetic system of the data, which no change to the tool can alter (the maintainer, 2026-10-10). The tool's own arithmetic is held to those rules by tests in CI.
  - What is left to the eye is what only an eye judges, on the tool alone: that the base map names places in Japanese, that the meshes lie where the base map draws land, which no program checks, and that panning and zooming stay smooth.
- The tool is checked on macOS only rather than made to show Japanese text everywhere, because it is a personal tool on a macOS machine, and a font shipped with it would be a file to add and a licence to check.
  - Without the font it shows no map rather than a map whose source statements cannot be read, because the roadmap makes stating the data's source a condition of showing the data.

## Dependencies

- **Rust toolchain**: builds and tests the tool; declared in `mise.toml` beside the tools already there.
- **`eframe`, `egui`, and `walkers`**: the window and controls, and the map widget with its tile fetching ([0002]).
- **A CSV reader**: parses the file; the crate is chosen while building.
- **`thiserror`**: derives the tool's error types.
- **`colorous`**: the colours of the scale, from a published colour scheme.
- **`fontdb`**: finds the font for Japanese text among the fonts installed on the system, by the name of its family.
- **`unit-prefix`**: writes a large value of the legend with a prefix such as k or M in place of its zeros.
- **`egui_kittest`**, for tests only: drives the window without showing it (A007).
- **`proptest`**, for tests only: makes test input in code, text in a Japanese script included, and checks a property over all of it.
- **Rust checks in `mise.toml` and CI**: formatting, lints, and tests join the tasks that gate a commit and a pull request.
  - CI stays on Linux and runs the tests that open no window; the tests that drive the window run on macOS, from the same task, on the maintainer's machine.

## Done when

- **For each of the three files, the tool holds the meshes, the choices, and every value as a second reading of the file finds them, one that shares no code with the tool's own reading, draws every mesh once, and offers that file's indicators, the total first.** — verifies R001, R002, A001.
  - Check: an automated test that reads the real files, run on the maintainer's machine when asked for. The evidence is the run, with no value.
- **With the Chukyo metropolitan area's file open, the base map names places in Japanese under the meshes, the meshes lie where the base map draws land, with none out at sea and no steady offset along the coast, and panning and zooming stay smooth.** — verifies R001, R005, A002, A003, A005, A006.
  - Check: the maintainer looks at the tool.
- **Mesh codes turn into the squares the standard defines, files that differ in indicators and in the byte order mark load, and zero gets the neutral colour with the two ends at plus and minus a high percentile of the absolute values shown.** — verifies R001, R002, R003, A001.
  - Check: automated tests on synthetic data, passing in CI.
- **Choosing another indicator recolours the map and changes the legend, pointing at a mesh shows its code, its municipality, and its value, and the base map shows through the meshes more or less as the user sets it.** — verifies R002, R003, R004, R005, A007.
  - Check: an automated test that drives the window with a synthetic file whose values are known from how it was made, passing on macOS.
- **The data's source, the base map's source with its link, and the credit to the shoreline data are on screen whenever the map is.** — verifies R006.
  - Check: an automated test that reads the window's text with a synthetic file open, passing on macOS.
- **Started with no path, with a path that does not exist, and with a file that is not Urban QOL data, the tool prints which file it expects and where it comes from, and exits without a map.** — verifies R007.
  - Check: an automated test for the three starts, passing in CI.
- **Started where the font cannot be loaded, the tool says so and exits without a map.** — verifies R006, R011.
  - Check: an automated test, passing in CI.
- **The README says how to get the file and start the tool, that only macOS is checked, and what happens without the font.** — verifies R007, R011.
  - Check: the README read.
- **With the network off, the meshes are drawn over a blank base map, and the tool makes no request to the platform.** — verifies R008.
  - Check: an automated test that draws a synthetic file with a tile server that cannot be reached, passing on macOS, and each place the code names the platform's host read to be text the tool shows, not an address it requests.
- **The tool leaves no file behind that holds anything from the CSV, and the repository holds no data: the tests that run on synthetic data make their input in code, the test on real files takes them from outside the repository, and no commit of the phase carries a picture or a value from a real file.** — verifies R009, R010.
  - Check: the code read for what it writes, and the check before each push that `CLAUDE.md` asks for.

## Open questions

None.

[0001]: ../decisions/0001-csv-kept-on-disk-as-data-source.md
[0002]: ../decisions/0002-native-rust-app-on-egui-and-walkers.md
