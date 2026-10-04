# Roadmap

## Purpose

The [Urban QOL data](https://data-platform.mlit.go.jp/#/Page?id=dataintro01) scores 500 m meshes in five metropolitan areas of Japan for how good each is to live in.
The score is a sum over indicators, such as how near a railway station is or how high the flood risk is, each weighted by how much the adults answering a 2021 survey valued it on average.
A person whose priorities differ from that average cannot tell from the published map how the same places would score for them.
QOL Reweight lets them say how many times the survey average they value each indicator, recomputes the score of every mesh, and shows the result on a map.

## Scope

- **Reweighting**: the user gives each indicator a multiplier, and a mesh's score is the sum of its published indicator values, each times its multiplier.
  - A multiplier is all a user can set, because the published values are already weighted and converted to yen per year, and the platform publishes neither the unweighted values nor the weights.
  - With every multiplier at 1, the score is the sum of the published indicators, which is the published total wherever that total is such a sum.
  - Recomputing on the user's own machine changes how the data is viewed and leaves the data as published, so the maintainer reads it as outside what the provider's terms call using the data altered. The licence itself [lets adapted material be made and withholds only sharing it](https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode).
- **Data**: one of the CSV files the platform publishes, a prefecture's or a metropolitan area's, which the user downloads from the platform and keeps on their machine [0001].
  - That a metropolitan area's file has the layout of a prefecture's is inferred from the files' sizes, and no such file has been read (2026-10-04).
  - The indicators are read from the file, since a file carries 16 or 26 of them depending on its region (the platform's catalogue, checked 2026-10-04).
  - A file whose published total is not the sum of the indicators it carries is covered too. Tokyo's is one, and Aichi's total is the sum (both files, checked 2026-10-04).
  - Nothing that comes from the data is committed to this repository or published, as [`CLAUDE.md`](../CLAUDE.md#keeping-the-data-out-of-the-repository) spells out.
  - Wherever the tool shows the data, it states the data's source, which the provider's terms make a condition of use.
- **Viewing**: a map of 500 m meshes on the user's own machine, which follows the multipliers as the user moves them.

## Non-goals

- **Publishing a reweighted result, or hosting the tool for others**: the data is licensed CC BY-NC-ND 4.0 and its provider's terms forbid using it altered without permission.
- **Indicators the published data leaves out**: rent, energy performance, and distance to a garbage collection point are part of the provider's method, and the platform's page says its data does not consider them. The two files read carry no indicator by those names (Tokyo's and Aichi's, checked 2026-10-04).

## Phases

- **Published values on a local map**: on the user's machine, one file's published values are shown on a map of 500 m meshes, the total or one indicator at a time.
- **Reweighting**: the user sets a multiplier for each indicator, and the map follows.
- **Several files at once**: the map covers more than one file, such as two neighbouring prefectures.
- **Totals that are not the sum**: a file whose published total is not the sum of its indicators is handled in a way its user can see.
- **Multipliers from the paper's weights**: choosing personal attributes, such as sex and age band, fills in the multipliers from the weight parameters that [the paper behind the data](https://doi.org/10.1016/j.cities.2023.104561) publishes by attribute.
  - They are approximate, because a multiplier is a ratio to the average weight the published values use, and how that average was taken is not in the part of the paper read (pages 1 to 12 of 19, 2026-10-04).

Published values on a local map comes first and Reweighting second, because the first settles the path from the file to the map against a picture the platform already shows, and the second then adds only what is this tool's own.
The order of the other three is decided when Reweighting ends.

[0001]: decisions/0001-csv-kept-on-disk-as-data-source.md
