# Roadmap

## Purpose

The [Urban QOL data](https://data-platform.mlit.go.jp/#/Page?id=dataintro01) scores 500 m meshes in five metropolitan areas of Japan for how good each is to live in.
The score is a sum over indicators, such as how near a railway station is or how high the flood risk is, each weighted by how much the adults living in that metropolitan area value it on average, as estimated from a 2021 survey.
A person whose priorities differ from that average cannot tell from the published map how the same places would look to them.
QOL Reweight lets them say how many times that average they value each indicator, and colours every mesh on a map by the sum of its published indicator values, each times that multiplier.

## Scope

- **Reweighting**: the user gives each indicator a multiplier, and the map colours a mesh by the sum of its published indicator values, each times its multiplier.
  - A multiplier is all a user can set, because the published values are already weighted and converted to yen per year.
  - With every multiplier at 1, that sum is the sum of the published indicators, which is the published total wherever that total is such a sum.
  - The tool reads the files, changes nothing in them, and writes no sum out, so the maintainer reads what it does as changing how the data is viewed, outside what the provider's terms call using the data altered. The licence itself [lets adapted material be made for non-commercial purposes and withholds sharing it](https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode).
- **Data**: the CSV files the platform publishes, one for each prefecture and each metropolitan area, which the user downloads from the platform and keeps on their machine [0001].
  - The indicators are read from each file, since the files differ in which indicators they carry.
  - A file whose published total is not the sum of the indicators it carries is covered too.
  - Nothing that comes from the data is committed to this repository or published, as [`CLAUDE.md`](../CLAUDE.md#keeping-the-data-out-of-the-repository) spells out.
  - Wherever the tool shows the data, it states the data's source, which the provider's terms make a condition of use.
- **Viewing**: a map of 500 m meshes on the user's own machine, which follows the multipliers as the user moves them.

## Non-goals

- **Publishing a reweighted map, or hosting the tool for others**: the data is licensed CC BY-NC-ND 4.0 and its provider's terms forbid using it altered without permission.
- **Writing the sums out to a file**: the tool changes how the published values are viewed and produces no data of its own.
- **Indicators the published data leaves out**: rent, energy performance, and distance to a garbage collection point are part of the provider's method and are not in the platform's data.
- **Reproducing the provider's scores for one sex, age band, or other attribute**: for an indicator measured as an access time the value itself depends on the attribute, as [the paper behind the data](https://doi.org/10.1016/j.cities.2023.104561) sets out, so multipliers on the published values cannot turn them into an attribute group's scores.

## Phases

- **Published values on a local map**: on the user's machine, one file's published values are shown on a map of 500 m meshes, the total or one indicator at a time.
- **Reweighting**: the user sets a multiplier for each indicator, and the map follows.
- **Several files at once**: the map covers more than one file, such as two neighbouring prefectures.
- **Totals that are not the sum**: a file whose published total is not the sum of its indicators is handled in a way its user can see.

Published values on a local map comes first and Reweighting second, because the first settles the path from the file to the map against a picture the platform already shows, and the second then adds only what is this tool's own.
The order of the other two is decided when Reweighting ends.

[0001]: decisions/0001-csv-kept-on-disk-as-data-source.md
