# Phase 02: Reweighting

- Status: Closed 2026-10-11
- Roadmap: [Reweighting](../ROADMAP.md#phases)

## Goal

With one file open, the user gives each of its indicators a multiplier, and the map colours every mesh by its weighted sum: the sum of its published indicator values, each times its multiplier.
The map follows a multiplier while it is being moved, and with every multiplier at 1 it shows the published total wherever that total is the sum of the file's indicators.

## Requirements & Constraints

- **R001**: The list of what the colours show has the weighted sum first, then the published total and each indicator as before, and the tool opens showing the weighted sum.
- **R002**: With every multiplier at 1, a mesh's weighted sum is the sum of the file's indicator rows for it, which is its published total in a file whose `QOL` row is that sum.
- **R003**: For each indicator of the file, the screen has a slider with a number field beside it that sets the indicator's multiplier, a tenth from 0 to 10, which is 1 when the tool opens.
  - The slider runs over the logarithm of the multiplier plus 1: 0 is at its left end and 10 at its right end, and a tenth more moves the handle a little less the larger the multiplier is, so the tenths next to 0 are the furthest apart and a slider at 0 can be told from one at 0.1 by where its handle is.
  - A number can be typed into the field, and one outside the range becomes the nearest end.
- **R004**: One button sets every multiplier to 1, and another sets every multiplier to 0.
- **R005**: While the weighted sum is shown, the colours and the legend follow a multiplier as it is moved, a button included, and the value read out for the mesh pointed at (R008) is its weighted sum under the multipliers as they then are.
  - The scale is fitted anew to the weighted sums at each change, as it is fitted to any values shown, so the colours tell where a mesh stands among the file's meshes whatever the multipliers are.
  - A weighted sum is read out in the form the files write their values in (A005), to ten significant figures with no zeros after the last of them, and a published value as the file writes it, as before.
  - Nothing is read out while the pointer is off the map, as it is while a slider is dragged, so a mesh's new sum is read once the pointer is back on the mesh.
- **R006**: While the published total or one indicator is shown, the sliders, the fields, and the buttons cannot be used and look so.
  - The multipliers keep their values, and the weighted sum has them when it is chosen again.
- **R007**: The sliders and the buttons are in the panel on the left, under what it had before them, and the map is as wide as it was before them.
  - What keeps its height comes over what does not: the legend, the pull-down, the slider for the opacity, and the two buttons, and under them the list of sliders, which is as long as the file has indicators and is the one thing in the panel whose height the file sets.
  - The list takes the room that is left and scrolls by itself where it is higher than that room, and what is over it stays in view meanwhile.
  - The panel keeps its size whatever the file holds, however many indicators and however long their names.
- **R008**: The code of the mesh pointed at, the name of its municipality, and the value shown are read out beside the pointer, on the map, and no longer in the panel.
  - Each is read out in full however long it is, on as many lines as it takes, and neither the panel nor the map's rectangle changes with it.
- **R009** (constraint): The tool still reads the file and writes nothing.
  - The multipliers are not kept from one run to the next, and no weighted sum leaves the tool.
- **R010** (constraint): Nothing that comes from the data is committed, as [`CLAUDE.md`](../../CLAUDE.md#keeping-the-data-out-of-the-repository) spells out, so the tests run on synthetic data and the phase's evidence holds no picture and no value.
  - A test that needs a real file takes it from outside the repository, runs only when asked for, and holds no value: what it expects, and how far two sums may differ, is read from the file.

### Out of scope

- A file whose `QOL` row is not the sum of its indicator rows is a later phase of the roadmap.
  - Here its weighted sum is worked out like any other file's, so with every multiplier at 1 it differs from the published total, which stays in the list beside it, and the tool does not say that the two differ.
- Several files at once, which is a later phase of the roadmap.
- A multiplier below 0, which would turn an indicator's sign over rather than say how many times its average weight the user gives it.
- Hiding the panel to give the map the whole window.
- In what is read out for a mesh, the published total beside the weighted sum, and what each indicator adds to the sum.
- A mesh's value kept on screen while the pointer is on the panel, such as by holding the last mesh pointed at or by pinning one with a click.
- Setting one indicator's multiplier back by itself, other than by typing 1.

## Assumptions & Risks

- **A001**: In the files of Aichi and of the Chukyo metropolitan area, every mesh's `QOL` row is the sum of its indicator rows, to within the rounding of the figures the file writes, and in Tokyo's file it is so for no mesh. Source: a trial script over the three files, run 2026-10-10.
  - Risk: a file's total is the sum in some meshes and not in others, so no check of R002 fits it, noticed by the test on real files failing on a file for some meshes alone.
- **A002**: The weighted sums of a file's meshes can be worked out and the scale fitted to them on every frame in which a multiplier moves. Source: the measurement in [0002], which coloured squares by a weighted sum of 26 values on every frame, and a later one, in which colouring 50,000 synthetic values anew, the scale's fitting included, took 1.4 ms in a release build (run 2026-10-08).
  - Risk: with a real file the map falls behind the slider, noticed by a map that stutters or trails while a slider is dragged with the Chukyo metropolitan area's file open.
- **A003**: egui's slider does what R003 asks without code of the tool's own for the scale: a logarithmic range from 1 to 11 for the multiplier plus 1, a step, a number field that writes and reads the multiplier itself through the slider's own formatter and parser, and clamping to the range. Source: [`slider.rs` of egui 0.36.2](https://github.com/emilk/egui/blob/0.36.2/crates/egui/src/widgets/slider.rs), read 2026-10-10, and the slider tried in the tool's window, 2026-10-11.
  - Risk: the slider does not behave so once drawn, noticed by a test of R003 that cannot be made to pass on egui's slider alone.
  - Risk: the handle for 0.1 is drawn about a twenty-fifth of the slider's length from the handle for 0, by the arithmetic of the scale, which may be too little to see, noticed by a slider at 0 that cannot be told from one at 0.1 at a glance.
- **A004**: Two lines for each indicator, its name over its slider and field, fit the panel's width, and about thirteen indicators are in view at once in the window as it opens. Source: an estimate from egui's default sizes, not drawn.
  - Risk: far fewer are in view, noticed by a list that shows only a few indicators in the window as it opens.
  - Risk: a name's line or a slider with its field does not fit the panel as wide as it is, noticed by a line that is cut off or a panel that has to be widened to hold it.
- **A005**: The files write each value with at most ten significant figures, with no zeros after the last of them and with no exponent. Source: a trial script over the files of Aichi, Tokyo, and the Chukyo metropolitan area, run 2026-10-10.
  - Risk: a file writes its values with more figures, so its published values and its weighted sums are read out in two forms, noticed by the test on real files finding a value of more than ten figures.
- **A006**: egui draws beside the pointer something the tool lays out, with no delay, and a test that drives the window can read its text. Source: [`tooltip.rs`](https://github.com/emilk/egui/blob/0.36.2/crates/egui/src/containers/tooltip.rs) and [`response.rs`](https://github.com/emilk/egui/blob/0.36.2/crates/egui/src/response.rs) of egui 0.36.2, which have a tooltip that is always open and one placed at the pointer, read 2026-10-10. Not tried in a window, and that a test can read it is unverified.
  - Risk: a test cannot find what is read out, noticed by a check of R008 that ends up done by hand.
  - Risk: what is read out lags behind the pointer or flickers as it passes from mesh to mesh, noticed by looking at the tool.

## Decisions

- The weighted sum is a choice of its own in the list rather than taking the published total's place, because the published total then stays on screen to compare with, and a file whose total is not the sum (A001) still shows what it publishes.
  - The tool opens on the weighted sum rather than on the published total, because that sum is what the tool is for, and opened on the total, moving a multiplier would change nothing on the map.
- The scale is fitted to the weighted sums as the multipliers then stand rather than held where every multiplier at 1 puts it, because the roadmap's purpose is how the same places look to the user, which is a comparison between places, and a held scale leaves most meshes at one end's colour or near the neutral one once the multipliers are far from 1.
  - So the colours depend on the ratios between the multipliers, but for the rounding of the scale's ends to two figures, and the legend's ends say how large the sums are.
- The map is coloured anew at each change of a multiplier, while a slider is still held, rather than when the slider is let go, because of everything the design covers the maintainer weighs most how the map follows a multiplier while it is being moved ([0002], under what the maintainer said).
- A multiplier runs from 0 to 10 on a slider over the logarithm of the multiplier plus 1, rather than on a logarithmic slider with 1 at its middle, or from 0 to 3 on a linear one, because the maintainer, having used the logarithmic one in the tool, its scale starting at about 0.09 and then at 0.05, found it unnatural next to 0, where the step from 0.1 to 0.2 was far longer than the step from 0 to 0.1 (said 2026-10-11), and the logarithm of the multiplier plus 1 has a place for 0 of itself.
  - 1 is then about three tenths of the way along the slider rather than at its middle, which the maintainer did not find unnatural, the most a multiplier can be being as large as 10 (said 2026-10-11).
  - Equal ratios are then not equal distances, which the decision this one replaces held to: a logarithmic slider from 0 to 10 with 1 at or close to its middle, its scale starting at about 0.09 so that 0 had the left end to itself (2026-10-10).
    - On such a slider any start of the scale puts either 0.1 against 0 or 1 away from the middle.
- Two buttons, every multiplier to 1 and every multiplier to 0, rather than the first alone, because the maintainer asked for the second (said 2026-10-10).
- The multipliers are not saved, because the tool writes nothing ([architecture](../ARCHITECTURE.md#invariants)), and the maintainer agreed to start from 1 on every run (said 2026-10-10).
- The sliders go into the panel on the left, two lines an indicator in a list that scrolls, rather than into a second panel on the right with every indicator in view, or a window floating over the map, because the maintainer wants the map as large as it can be (said 2026-10-10), and a floating window covers part of the map while a slider is moved.
- A weighted sum is read out to ten significant figures rather than with every figure the arithmetic leaves, or as a whole number, because that is the form the files write their values in (A005), which the maintainer asked the sum to be held to (said 2026-10-10), so the sum and the published total are read in one form.
- The list of sliders is the last thing in the panel, and the one thing in it that varies in height, because the maintainer wants what varies in length under what does not, as far as that can be (said 2026-10-10).
- What the file says of the mesh pointed at is read out beside the pointer rather than in the panel, whether left where it was, moved under the list, or cut to a line each, because a long name makes a readout in the panel higher and moves what is under it, a line cut short could not be read in full by pointing at it, since the pointer would leave the mesh to do so, and the maintainer prefers it beside the pointer, as nearer to how the platform behaves (said 2026-10-10).
  - It takes the place of the panel's readout rather than standing beside it, because the two would say the same thing at the same time.
- While the weighted sum is not shown the sliders are disabled, rather than left as they are or made to switch the map to the weighted sum when moved, because a control that changes nothing on the map should look so, and a switch would replace what the user chose to look at.
- What the screen calls the weighted sum, the multipliers over their list, and the two buttons, in Japanese, the maintainer chose as the pull request that adds each was shown (2026-10-10 and 2026-10-11), and [`GLOSSARY.md`](../GLOSSARY.md) pairs each with its English name.
- That the map follows a slider smoothly is checked by eye rather than by a test that times the work, because nothing gives such a test a limit to hold the time to, and the time differs between machines.

## Dependencies

None.

## Done when

- **For each real file, the weighted sum the tool holds for every mesh with every multiplier at 1 is what R002 says it is; each file's `QOL` rows are those sums in every mesh or in none, and in every mesh of at least one file; and no value of a file has more figures than A005 says.** — verifies R001, R002, A001, A005.
  - Check: an automated test that reads the real files, run on the maintainer's machine when asked for, over the files of Aichi, Tokyo, and the Chukyo metropolitan area.
    - It sets what the tool holds against a second reading of the file, one that shares no code with the tool's own reading.
    - The evidence is the run and which files' totals it found to be the sum, with no value.
  - Evidence: `QOL_REWEIGHT_REAL_FILES=<folder> cargo test a_real_file -- --ignored --show-output` on macOS, 2026-10-11, on `main` at `042eb98`, over the files of Aichi, Tokyo, and the Chukyo metropolitan area: `a_real_files_weighted_sums_at_1_are_the_sums_of_its_indicator_rows` in `src/multipliers.rs` passes, with `a_real_file_is_offered_and_drawn_as_it_reads` in `src/window.rs`.
    - It reports the total to be the sum of the indicators in every mesh of Aichi's file and of the Chukyo metropolitan area's, and in no mesh of Tokyo's.
    - Its second reading uses the CSV reader alone and none of the tool's `src/dataset/`, and how far two sums may differ it works out from the figures each file writes.
- **The tool opens as R001 says, with every multiplier at 1; setting a multiplier by each control of R003 and R004 recolours the meshes and changes the legend to those of the new weighted sums, and a mesh pointed at afterwards has its sum read out in the form R005 gives, whatever figures the arithmetic leaves.** — verifies R001, R002, R003, R004, R005, A003.
  - Check: an automated test that drives the window with a synthetic file open, passing on macOS.
    - It works each weighted sum out from how the file was made.
  - Evidence: `mise run qa:rust:test` on macOS, 2026-10-11, on `main` at `042eb98`: 104 unit tests and 5 start-up tests pass.
    - In `src/window.rs`:
      - `the_choices_are_the_weighted_sum_the_total_then_the_indicators_of_the_file`
      - `the_window_opens_on_the_weighted_sum_of_the_indicators`
      - `setting_a_multiplier_colours_the_meshes_by_the_new_weighted_sums`, which sets a multiplier by a slider, by a number typed, and by each button
      - `a_weighted_sum_is_read_out_to_ten_figures`
      - `the_map_follows_a_slider_while_it_is_held`
- **A slider has the ends, the steps, and the wider spacing next to 0 that R003 states, and a number typed beyond the range becomes the nearest end.** — verifies R003, A003.
  - Check: an automated test that drives the window, passing on macOS.
    - It sets where a slider draws its handle for 0.1 against where it draws it for 0 and for 0.2: to the right of the first, and further from it than the last is from the handle for 0.1.
  - Evidence: the same run: `a_multipliers_slider_runs_from_0_to_10_by_tenths` in `src/window.rs`.
- **With the published total or an indicator chosen, the controls R006 names take no input and the map does not change; with the weighted sum chosen again, the map is as the multipliers set before left it.** — verifies R006.
  - Check: an automated test that drives the window, passing on macOS.
  - Evidence: `the_multipliers_cannot_be_set_while_the_weighted_sum_is_not_shown` in `src/window.rs`, run on macOS, 2026-10-11, on the commit that adds this line, which also has the test choose an indicator after the total: it passes.
- **The map's rectangle is the one the tool gave it before the phase, whatever the file holds; the last slider of a list longer than its room can be scrolled to and moved, and the legend, the pull-down, and the buttons stay where they were.** — verifies R007, A004.
  - Check: an automated test that drives the window with generated text, passing on macOS.
    - It opens a file of a few indicators and one of many more indicators than the list has room for, whose names are far longer than the panel is wide.
    - The rectangle it finds is set against the one the tool draws at the commit the phase started from.
  - Evidence: the same run: `the_multipliers_leave_the_map_where_it_was_and_scroll_by_themselves` in `src/window.rs`, with a file of two indicators and with one of forty-one.
    - The rectangle it holds the map to is the one a run of the tests at `dde0ac2`, before the phase, found the map to have in the same window.
- **Pointing at a mesh reads out what R008 lists beside the pointer and nowhere in the panel; a long name and a long value are read out in full and change nothing, as R008 says; and nothing is read out with the pointer off the meshes, on the panel, or on an open pull-down that lies over a mesh.** — verifies R008, A006.
  - Check: an automated test that drives the window with generated text, a name and a value far longer than the panel is wide, passing on macOS.
  - Evidence: the same run.
    - In `src/window.rs`:
      - `pointing_at_a_mesh_reads_out_its_code_its_municipality_and_the_value_shown`, which finds each text once in the window, near the pointer
      - `a_long_name_and_a_long_value_are_read_out_in_full_and_move_nothing`
      - `pointing_away_from_the_meshes_reads_out_nothing`
      - `pointing_at_the_panel_over_a_mesh_reads_out_nothing`
      - `pointing_at_a_choice_over_a_mesh_reads_out_nothing`
- **The map follows a slider while it is dragged, without stutter and without trailing behind it, about thirteen indicators are in view in the window as it opens, a slider at 0 is told from one at 0.1 at a glance, and what is read out beside the pointer keeps up with it from mesh to mesh.** — verifies R003, R005, R008, A002, A004, A006.
  - Check: the maintainer looks at the tool with the Chukyo metropolitan area's file open.
  - Evidence: the maintainer opened the Chukyo metropolitan area's file in a release build on 2026-10-11, with no picture kept, and found the map to follow a slider while it is dragged, sixteen indicators to be in view in the window as it opens, where A004 estimated about thirteen, and what is read out to keep up with the pointer.
    - A slider at 0 could be told from one at 0.1 on the scale the plan then had, which the maintainer found unnatural next to 0; the scale was changed to the one R003 now states, the maintainer chose it looking at the tool the same day, and on it too found a slider at 0 to be told from one at 0.1 at a glance.
- **The tool still leaves no file behind, and no commit of the phase carries a picture or a value from a real file.** — verifies R009, R010.
  - Check: the code read for what it writes, and the check before each push that `CLAUDE.md` asks for.
  - Evidence:
    - A search of `src` for a call that creates or writes a file finds none outside a test that writes CSV into memory.
    - The phase changes neither `Cargo.toml` nor `Cargo.lock`, so eframe's `persistence` feature is still left out and no state of the window, the multipliers included, is saved.
    - The tests the phase adds make their input in code, the test on real files reads them from the folder it is given and prints no value, and each push of the phase was preceded by the check `CLAUDE.md` asks for.

## Open questions

None.

[0002]: ../decisions/0002-native-rust-app-on-egui-and-walkers.md
