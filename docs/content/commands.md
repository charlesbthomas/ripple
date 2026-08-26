+++
title = "Commands"
description = "Every ripple command, with flags and examples"
+++

Every command documents itself: `ripple <command> --help` includes worked examples.

## `ripple changed`

Lists modules affected by a diff: modules whose files changed (**direct**) plus every module that transitively depends on one (**indirect**).

The default diff is your working tree — including uncommitted and untracked files — against the merge-base of HEAD and the base branch. That matches what a pull request from your branch would contain.

```sh
ripple changed                   # working tree vs merge-base with main
ripple changed --base develop    # different base branch
ripple changed main...HEAD       # merge-base diff between two refs (CI)
ripple changed HEAD~3..HEAD      # literal diff between two refs
ripple changed --staged          # staged changes only (pre-commit)
ripple changed --direct-only     # skip transitive dependents
ripple changed --strict          # fail if any changed file has no owning module
```

### Diff selection

| Form | Meaning |
|------|---------|
| *(none)* | Working tree vs merge-base of HEAD and the base branch |
| `A...B` | Merge-base diff between two refs |
| `A..B` | Literal diff between two refs |
| `--base <ref>` | Different base branch (default: `base` from ripple.toml, else `main`) |
| `--staged` | Staged changes only |

### Output formats

`--format` accepts:

- `table` — aligned columns with status and dependency chain (default on a TTY)
- `plain` — one module name per line (default when piped)
- `json` — the full report, machine-readable
- `github` — a GitHub Actions matrix include list

Changed files that no module owns are reported on stderr; `--strict` turns them into an error.

## `ripple validate`

Checks the config: parse errors, duplicate module names, unknown dependencies (with nearest-name suggestions), dependency cycles, missing paths, and overlapping path warnings. Non-zero exit on any error.

```sh
ripple validate
```

## `ripple graph`

Inspects the module dependency graph. With no arguments, lists every module with its paths and dependencies. With a module, walks the graph from it.

```sh
ripple graph                      # list all modules and their deps
ripple graph web --deps           # everything web depends on, as a tree
ripple graph core --dependents    # everything a core change ripples to
ripple graph --format mermaid     # whole graph for a markdown doc
ripple graph --format dot | dot -Tsvg > graph.svg
```

## `ripple explain`

Shows why a module is, or is not, affected: the exact matched files for a direct change, or the dependency chain for an indirect one. Accepts the same diff selection as `ripple changed`.

```sh
ripple explain api                # why is api affected right now?
ripple explain web main...HEAD    # same, for an explicit range
```

## `ripple init`

Writes a commented starter `ripple.toml` in the current directory. Refuses to overwrite an existing file.

## `ripple completions`

Generates a completion script for your shell on stdout:

```sh
ripple completions zsh > "${fpath[1]}/_ripple"
ripple completions bash > /etc/bash_completion.d/ripple
ripple completions fish > ~/.config/fish/completions/ripple.fish
```

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success, including "no modules affected" |
| `1` | Error: bad config, git failure, `--strict` violation |
| `2` | Usage error |
