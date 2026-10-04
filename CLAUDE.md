# CLAUDE.md

QOL Reweight recomputes the Urban QOL data published on the [MLIT Data Platform](https://data-platform.mlit.go.jp/#/Page?id=dataintro01) with weights its user chooses, and shows the result on a map of 500 m meshes.
It is a local tool for personal use, and its code is public.

## Development documents

Development is steered through the documents the [`dev-docs`](https://github.com/irisTa56/dotfiles/blob/main/.claude/skills/dev-docs/SKILL.md) skill describes, in that skill's default layout under `dev/`.
Follow that skill when starting, working in, or closing a phase, and when recording a decision.

## Keeping the data out of the repository

The Urban QOL data is licensed [CC BY-NC-ND 4.0](https://creativecommons.org/licenses/by-nc-nd/4.0/), and its provider's terms of use, on the page linked above, forbid using it altered without permission.
This repository is public, and [a commit pushed to GitHub stays reachable](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository) after its branch is rewritten or deleted.
So nothing that comes from the data is committed on any branch or published, in a file, a commit message, a pull request, or an issue, for example:

- the source data, as a CSV file or as vector tiles
- recomputed values, and a map image drawn from the source data or from recomputed values
- a screenshot of the tool showing real data, such as one kept as evidence that a phase is done
- a value taken from real data, such as a row used as a test fixture or a threshold used to colour the map

A reweighted result is not published anywhere else either.

What the platform publishes about the data is not the data, so naming it is fine: an indicator's name, a region's name, a file's name and size, and the formula the values follow.

Tests run on synthetic data, made up for the purpose and not sampled from the real data.

Research notes and downloaded data may hold such values, which is why they are kept in the private workspace the `dev-docs` skill describes, outside this repository's working tree.
Before every push, check that no commit it would publish carries anything of this kind, in its tree or its message, and rewrite those commits first if one does.

## Commands

[`mise.toml`](mise.toml) is the task list and carries its own reasons.
`mise install` installs the tools and the git hooks, and `mise run pre-commit` is the gate the pre-commit hook runs.

## Git workflow

- Never push to `main` directly; branch first, then open a pull request, which is squash-merged.

## Delegation

- Pass `run_in_background: true` explicitly on every background Agent call, although it is the default.
  - Entire records a subagent's transcript only when that argument is passed ([entireio/cli#2556](https://github.com/entireio/cli/issues/2556)).
  - The rule applies only where the Agent tool has that parameter, which [fork mode](https://code.claude.com/docs/en/sub-agents#turn-fork-mode-on-or-off) removes; in such a session, tell the user that subagents are not recorded.

## Writing conventions

- Everything committed is written in English: code, comments, documents, and commit messages.
- Prose follows [`document-writing.md`](https://github.com/irisTa56/dotfiles/blob/main/.claude/rules/document-writing.md).
