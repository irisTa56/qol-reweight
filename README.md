# QOL Reweight

QOL Reweight shows the Urban QOL data on a map of 500 m meshes, on its user's own machine.
Its user says how much they value each indicator, as a multiple of what the adults living in the area value it on average, and each mesh is coloured by the sum of its published indicator values, each times that multiplier.

So far the tool shows the published values as they are, the total or one indicator at a time.
The multipliers are not built yet, and the [roadmap](dev/ROADMAP.md) says what is planned.

## Data

The tool reads the Urban QOL data, which this repository does not contain.
Its user downloads the data from the [MLIT Data Platform](https://data-platform.mlit.go.jp/#/Page?id=dataintro01), which publishes it under [CC BY-NC-ND 4.0](https://creativecommons.org/licenses/by-nc-nd/4.0/) and its provider's terms of use.

The platform publishes one CSV file for each prefecture and each metropolitan area, such as `QOL_23_Aichi.csv`, and the tool opens one of them at a time.
Keep the file outside this repository's working tree, so that no commit can pick it up.

Source of the data: `都市QOLデータ2020（大日本ダイヤコンサルタント㈱作成）`

## Running the tool

The tool is checked on macOS only.

1. Install [mise](https://mise.jdx.dev/getting-started.html), and run `mise install` in this repository, which installs the Rust toolchain the tool is built with.
   It also installs the tools development uses and sets up this clone's git hooks, which run the repository's checks before a commit and a push.
2. Start the tool with the path of a downloaded file:

   ```sh
   cargo run --release -- path/to/QOL_23_Aichi.csv
   ```

A window opens with the file's meshes on a map, each coloured by its published total.

- The pull-down beside the map chooses what the colours show, the total or one indicator of the file, and the legend over it says which colour stands for which value.
- The slider under the pull-down sets how much of the base map the meshes cover.
- Pointing at a mesh shows its mesh code, the name of its municipality, and its value.
- Dragging the map or scrolling over it pans it, and pinching on a trackpad, or scrolling with the command or control key held, zooms it.

The base map is the [GSI tiles](https://maps.gsi.go.jp/development/ichiran.html), fetched over the network while the map is looked at.
That is the tool's only use of the network: it sends the platform no request, and without the network it draws the meshes over a blank base map.
It reads the file and writes nothing.

Started without a path, or with a file it cannot read as Urban QOL data, the tool prints which file it expects and where to download it, and shows no map.

### Without the font

The tool states the sources of the data and of the base map on screen in Japanese, in the font Hiragino Sans, which macOS has.
Where that font is not installed, as on Windows and Linux, the tool says so and shows no map.

## Development

Development documents, starting with the [roadmap](dev/ROADMAP.md) and the [architecture overview](dev/ARCHITECTURE.md), live under [`dev/`](dev/).

The tests run on made-up data.
One more test reads real files and checks what the tool holds and draws against a second reading of each; it runs only when asked for, on every CSV file in the folder a variable names:

```sh
QOL_REWEIGHT_REAL_FILES=path/to/folder cargo test a_real_file -- --ignored
```
