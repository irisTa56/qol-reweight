# 0003. Take BeautifulRust's lints for an application, and watch them upstream rather than follow them

- Status: Accepted
- Date: 2026-11-21

## Context

[BeautifulRust](https://github.com/morganlinton/BeautifulRust) ships the `rust-best-practices` skill, and with it the lints that check what of that skill a machine can check, as `lints/lints.toml` for `Cargo.toml` and `lints/clippy.toml` (both read at `b9718436`, checked 2026-11-21).

- `lints.toml` marks some of its lints as fitting a library and says an application can drop them: `missing_docs`, `missing_panics_doc`, `missing_errors_doc`, `exhaustive_enums`, `print_stdout`, and `print_stderr`.
  - The tool is a binary only, and `src/main.rs` writes its usage and its errors with `eprintln!`, which `print_stderr` would flag.
- Under the rest, with `clippy.toml` as it stands, the code drew warnings at 34 places (`cargo clippy --all-targets`, run 2026-11-21).
  - The most frequent were casts between floats and integers that the code means, which the lints ask to be stated with `#[expect]` and a reason.
- `qa:rust:clippy` runs clippy with `-D warnings` and is part of the gate every commit passes ([`mise.toml`](../../mise.toml)), so any lint added to the set fails commits until the code meets it.
- Cargo's `[lints]` table takes entries from elsewhere only from a workspace root's `[workspace.lints]`, not from a file such as `lints.toml` or from a URL ([the manifest's `lints` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section) and [the workspace's](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table), checked 2026-11-21).
- dotfiles' `upstream:pin-check`, which this repository's CI already runs, says when a path pinned in an upstream repository differs between the pinned commit and the upstream's default branch, and opens a pull request moving the pin ([its documentation](https://github.com/irisTa56/dotfiles/blob/main/tasks/README.md#upstreampin-check), checked 2026-11-21).

## Decision

`Cargo.toml` sets the lints of `lints/lints.toml` less the six it marks as fitting a library, and `clippy.toml` takes the settings of `lints/clippy.toml` whole.
A marker beside each pins its upstream file at the commit it was taken from, so `upstream:pin-check` reports a change to either file, and not to the rest of `lints/`, and the pull request it opens is where the change is taken into this repository's files by hand, with the code changed to meet it.

## Rejected alternatives

- **Taking the library lints too**: upstream marks them as fitting a library; here `print_stderr` would flag how the tool reports its errors, `missing_docs` would ask for documentation of the crate itself, and the other four flag nothing (`cargo clippy --all-targets` with the six added, run 2026-11-21).
- **Following the upstream files as they change**: Cargo cannot take the lints from the upstream file, and a copy refreshed by a job would turn each lint added upstream into failed commits until the code meets it, while the pull request `upstream:pin-check` opens leaves the timing to the maintainer.
- **A job that opens a pull request with the upstream files copied over this repository's**: it leaves the timing to the maintainer as well, but the copy here leaves out part of the upstream file, so a job writing the upstream files over it would put back the six lints left out unless it knew what to leave out: the lints are taken in a way no patch can be applied to automatically.
- **Only clippy's `pedantic` group, as dotfiles' own crates set it**: it leaves out the lints on panics and on suppressions, which are in clippy's `restriction` group, and `as_conversions`, and keeps the default under which `fn_params_excessive_bools` lets a function take three `bool` parameters (`clippy-driver -W help`, checked 2026-11-21), so a review would have to check those by reading, on the judgment, not checked against any source, that a rule checked by a machine is broken less than one checked by a reader.

## Consequences

- **Dependencies added**: none to the build; the lint set follows BeautifulRust's choices, and the pin makes `upstream:pin-check` fetch that repository.
- **Risks**:
  - A rule this repository disagrees with may arrive upstream; the bump pull request shows it, and the local copy may leave it out, which the pin does not prevent, since it compares the upstream with itself.
  - A clippy update can fail the gate as well, since the `all` and `pedantic` groups grow with it and the toolchain is not pinned (`mise.toml` installs the latest Rust); `upstream:pin-check` does not announce it.
  - `clippy.toml` sets `max-fn-params-bools = 0`, so a function with a `bool` parameter fails the gate, including one in a test.
