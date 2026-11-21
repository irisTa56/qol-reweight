# 0003. Take BeautifulRust's lints for an application, and watch them upstream rather than follow them

- Status: Accepted
- Date: 2026-11-21

## Context

The code is written with the `rust-best-practices` skill of [BeautifulRust](https://github.com/morganlinton/BeautifulRust) on, and the same repository ships the lints that check what of that skill a machine can check, as `lints/lints.toml` for `Cargo.toml` and `lints/clippy.toml` (both read at `b9718436`, checked 2026-11-21).

- `lints.toml` marks some of its lints as fitting a library and says an application can drop them: `missing_docs`, `missing_panics_doc`, `missing_errors_doc`, `exhaustive_enums`, `print_stdout`, and `print_stderr`.
  - The tool is a binary only, and `src/main.rs` writes its usage and its errors with `eprintln!`, which `print_stderr` would flag.
- Under the rest, with `clippy.toml` as it stands, the code had 34 warnings (`cargo clippy --all-targets`, run 2026-11-21).
  - The most frequent were casts between floats and integers that the code means, which the lints ask to be stated with `#[expect]` and a reason.
- `qa:rust:clippy` runs clippy with `-D warnings` and is part of the gate every commit passes ([`mise.toml`](../../mise.toml)), so any lint added to the set fails commits until the code meets it.
- Cargo's `[lints]` table has no way to take its entries from another file or from a URL ([the manifest's `lints` section](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section), checked 2026-11-21).
- dotfiles' `upstream:pin-check`, which this repository's CI already runs, says when a path pinned in an upstream repository differs between the pinned commit and the upstream's default branch, and opens a pull request moving the pin ([its documentation](https://github.com/irisTa56/dotfiles/blob/main/tasks/README.md#upstreampin-check), checked 2026-11-21).

## Decision

`Cargo.toml` sets the lints of `lints/lints.toml` less the six it marks as fitting a library, and `clippy.toml` is `lints/clippy.toml` as it stands.
A marker beside the lints pins `lints/` at the commit they were taken from, so `upstream:pin-check` reports a change there, and the pull request it opens is where the change is taken into this repository's files by hand, with the code changed to meet it.

## Rejected alternatives

- **Taking the library lints too**: `print_stderr` would flag how the tool reports its errors, and the rest check items exported from a library, which a binary has none of.
- **Following the upstream files as they change**: Cargo cannot take the lints from the upstream file, and a copy refreshed by a job would turn each lint added upstream into failed commits until the code meets it, while the pull request `upstream:pin-check` opens leaves the timing to the maintainer.
- **Only clippy's `pedantic` group, as dotfiles' own crates set it**: it leaves out the lints that check the skill's rules on panics, suppressions, casts and bool parameters, so a review would have to check those by reading, on the judgment, not checked against any source, that a rule checked by a machine is broken less than one checked by a reader.

## Consequences

- **Dependencies added**: none to the build; the lint set follows BeautifulRust's choices, and the pin makes `upstream:pin-check` fetch that repository.
- **Risks**:
  - A rule this repository disagrees with may arrive upstream; the bump pull request shows it, and the local copy may leave it out, which the pin does not prevent, since it compares the upstream with itself.
  - `clippy.toml` sets `max-fn-params-bools = 0`, so a function with a `bool` parameter fails the gate, including one in a test.
