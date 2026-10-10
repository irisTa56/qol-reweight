# Phase 02: Reweighting

- Status: In progress
- Roadmap: [Reweighting](../ROADMAP.md#phases)

## Goal

With one file open, the user gives each of its indicators a multiplier, and the map colours every mesh by its weighted sum: the sum of its published indicator values, each times its multiplier.
The map follows a multiplier while it is being moved, and with every multiplier at 1 it shows the published total wherever that total is the sum of the file's indicators.

## Requirements & Constraints

- **R001**: The list of what the colours show has the weighted sum first, then the published total and each indicator as before, and the tool opens showing the weighted sum.
- **R002**: With every multiplier at 1, a mesh's weighted sum is the sum of the file's indicator rows for it, which is its published total in a file whose `QOL` row is that sum.
- **R003**: For each indicator of the file, the screen has a slider with a number field beside it that sets the indicator's multiplier, a tenth from 0 to 10, which is 1 when the tool opens.
  - The slider is logarithmic: 1 is at its middle, 10 at its right end, 0.1 beside its left end, and 0 at that end.
  - A number can be typed into the field, and one outside the range becomes the nearest end.
- **R004**: One button sets every multiplier to 1, and another sets every multiplier to 0.
- **R005**: While the weighted sum is shown, the colours and the legend follow a multiplier as it is moved, a button included, and the value read out for the mesh pointed at is its weighted sum under the multipliers as they then are.
  - The scale is fitted anew to the weighted sums at each change, as it is fitted to any values shown, so the colours tell where a mesh stands among the file's meshes whatever the multipliers are.
  - The readout keeps its three lines and says nothing while the pointer is off the map, as it is while a slider is dragged, so a mesh's new sum is read once the pointer is back on the mesh.
- **R006**: While the published total or one indicator is shown, the sliders, the fields, and the buttons cannot be used and look so. The multipliers keep their values, and the weighted sum has them when it is chosen again.
- **R007**: The sliders and the buttons are in the panel on the left, between the slider for the opacity and the readout, and the map is as wide as it was before them.
  - The list of sliders scrolls by itself where it is higher than its room, and the legend, the pull-down, the buttons, and the readout stay in view meanwhile.
  - The panel keeps its size whatever the file holds, however many indicators and however long their names.
- **R008** (constraint): The tool still reads the file and writes nothing. The multipliers are not kept from one run to the next, and no weighted sum leaves the tool.
- **R009** (constraint): Nothing that comes from the data is committed, as [`CLAUDE.md`](../../CLAUDE.md#keeping-the-data-out-of-the-repository) spells out, so the tests run on synthetic data and the phase's evidence holds no picture and no value.
  - A test that needs a real file takes it from outside the repository, runs only when asked for, and holds no value: what it expects, and how far two sums may differ, is read from the file.

### Out of scope

- A file whose `QOL` row is not the sum of its indicator rows is a later phase of the roadmap. Here its weighted sum is worked out like any other file's, so with every multiplier at 1 it differs from the published total, which stays in the list beside it, and the tool does not say that the two differ.
- Several files at once, which is a later phase of the roadmap.
- A multiplier below 0, which would turn an indicator's sign over rather than say how many times its average weight the user gives it.
- Hiding the panel to give the map the whole window.
- In the readout, the published total beside the weighted sum, and what each indicator adds to the sum.
- A mesh's value kept on screen while the pointer is on the panel, such as by holding the last mesh pointed at or by pinning one with a click.
- Setting one indicator's multiplier back by itself, other than by typing 1.

## Assumptions & Risks

- **A001**: In the files of Aichi and of the Chukyo metropolitan area, every mesh's `QOL` row is the sum of its indicator rows, to within the rounding of the figures the file writes, and in Tokyo's file it is so for no mesh. Source: a trial script over the three files, run 2026-10-10.
  - Risk: a file's total is the sum in some meshes and not in others, so no check of R002 fits it, noticed by the test on real files failing on a file for some meshes alone.
- **A002**: The weighted sums of a file's meshes can be worked out and the scale fitted to them on every frame in which a multiplier moves. Source: the measurement in [0002], which coloured squares by a weighted sum of 26 values on every frame, and a later one of colouring 50,000 synthetic values anew, the scale's fitting included, in a release build (run 2026-10-08).
  - Risk: with a real file the map falls behind the slider, noticed by a map that stutters or trails while a slider is dragged with the Chukyo metropolitan area's file open.
- **A003**: egui's slider does what R003 asks without code of the tool's own for the scale: a logarithmic range that starts at zero, with the smallest value above zero set, a step, a number field that takes typed input, and clamping to the range. Source: [`slider.rs` of egui 0.36.2](https://github.com/emilk/egui/blob/0.36.2/crates/egui/src/widgets/slider.rs), read 2026-10-10. Not tried in a window.
  - Risk: the slider does not behave so once drawn, noticed by a test of R003 that cannot be made to pass on egui's slider alone.
- **A004**: Two lines for each indicator, its name over its slider and field, fit the panel's width, and about eleven indicators are in view at once in the window as it opens. Source: an estimate from egui's default sizes, not drawn.
  - Risk: far fewer are in view, noticed by a list that shows only a few indicators in the window as it opens.
  - Risk: a name's line or a slider with its field does not fit the panel as wide as it is, noticed by a line that is cut off or a panel that has to be widened to hold it.

## Decisions

- The weighted sum is a choice of its own in the list rather than taking the published total's place, because the published total then stays on screen to compare with, and a file whose total is not the sum (A001) still shows what it publishes.
  - The tool opens on the weighted sum rather than on the published total, because that sum is what the tool is for, and opened on the total, moving a multiplier would change nothing on the map.
- The scale is fitted anew at each change of a multiplier rather than held where every multiplier at 1 puts it, or fitted when the slider is let go, because the roadmap's purpose is how the same places look to the user, which is a comparison between places: a held scale leaves most meshes at one end's colour or near the neutral one once the multipliers are far from 1, and fitting on release changes every colour at once after the drag.
  - So the colours depend only on the ratios between the multipliers, and the legend's ends say how large the sums are.
- A multiplier runs from 0 to 10 on a logarithmic slider with 1 at its middle, rather than from 0 to 3 on a linear one, or from 0 to 10 as a logarithm of the multiplier plus 1, because the colours depend on ratios, so equal ratios should be equal distances, and the maintainer asked for a logarithmic 0 to 10 (said 2026-10-10).
  - Zero is at the slider's left end though no logarithm puts it there, because a multiplier of 0 leaves an indicator out, which the user must be able to say.
- Two buttons, every multiplier to 1 and every multiplier to 0, rather than the first alone, because the maintainer asked for the second (said 2026-10-10).
- The multipliers are not saved, because the tool writes nothing ([architecture](../ARCHITECTURE.md#invariants)), and the maintainer agreed to start from 1 on every run (said 2026-10-10).
- The sliders go into the panel on the left, two lines an indicator in a list that scrolls, rather than into a second panel on the right with every indicator in view, or a window floating over the map, because the maintainer wants the map as large as it can be (said 2026-10-10), and a floating window covers part of the map while a slider is moved.
- While the weighted sum is not shown the sliders are disabled, rather than left as they are or made to switch the map to the weighted sum when moved, because a control that changes nothing on the map should look so, and a switch would replace what the user chose to look at.
- That the map follows a slider smoothly is checked by eye rather than by a test that times the work, because nothing gives such a test a limit to hold the time to, and the time differs between machines.

## Dependencies

None.

## Done when

- **For each real file, the weighted sum the tool holds for every mesh with every multiplier at 1 is the sum of that mesh's indicator rows as a second reading of the file finds them, one that shares no code with the tool's own reading. The second reading finds each file's `QOL` rows to be those sums in every mesh or in none; where it is every mesh, the weighted sum is the published total as well, and at least one file is such a file.** — verifies R001, R002, A001.
  - Check: an automated test that reads the real files, run on the maintainer's machine when asked for, over the files of Aichi, Tokyo, and the Chukyo metropolitan area. The evidence is the run and which files' totals it found to be the sum, with no value.
- **With a synthetic file open, the list has the weighted sum first and the tool shows it; moving one indicator's slider, typing into its field, and pressing each button recolour the meshes and change the legend to those of the weighted sums the new multipliers give, and a mesh pointed at afterwards reads out its weighted sum under them; the test works each out from how the file was made.** — verifies R001, R002, R003, R004, R005, A003.
  - Check: an automated test that drives the window, passing on macOS.
- **A slider goes from 0 at its left end through 1 at its middle to 10 at its right end, stops at tenths, and a number typed beyond the range becomes the nearest end.** — verifies R003, A003.
  - Check: an automated test that drives the window, passing on macOS.
- **With the published total or an indicator chosen, the sliders, the fields, and the buttons take no input and the map does not change; with the weighted sum chosen again, the map is as the multipliers set before left it.** — verifies R006.
  - Check: an automated test that drives the window, passing on macOS.
- **The map's rectangle is the one the tool gave it before the phase, with a file of a few indicators and with one of many more indicators than the list has room for, whose names are far longer than the panel is wide; the last of them can be scrolled to and moved, and the legend, the pull-down, the buttons, and the readout stay where they were.** — verifies R007, A004.
  - Check: an automated test that drives the window with generated text, passing on macOS, and the rectangle it finds set against the one the tool draws at the commit the phase started from.
- **With the Chukyo metropolitan area's file open, the map follows a slider while it is dragged, without stutter and without trailing behind it, and about eleven indicators are in view in the window as it opens.** — verifies R005, A002, A004.
  - Check: the maintainer looks at the tool.
- **The tool still leaves no file behind, and no commit of the phase carries a picture or a value from a real file.** — verifies R008, R009.
  - Check: the code read for what it writes, and the check before each push that `CLAUDE.md` asks for.

## Open questions

- What the screen calls the weighted sum, the list of multipliers, and the two buttons, in Japanese. The maintainer chooses each when the pull request that adds it is shown.

[0002]: ../decisions/0002-native-rust-app-on-egui-and-walkers.md
