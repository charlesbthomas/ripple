# ripple

Monorepo change detection. Given a git diff, ripple reports which modules changed — both the modules whose files changed and every module that transitively depends on them.

Declare your dependency graph once in `ripple.toml`, then use one command locally and in CI to answer "what does this change affect?"

```
$ ripple changed
MODULE  STATUS    VIA
api     indirect  core
core    direct
web     indirect  api -> core

3 modules affected (1 direct, 2 indirect)
```

## Install

Homebrew (macOS and Linux):

```sh
brew install charlesbthomas/tap/ripple
```

Shell installer (prebuilt binaries):

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/charlesbthomas/ripple/releases/latest/download/ripple-installer.sh | sh
```

From source:

```sh
cargo install --git https://github.com/charlesbthomas/ripple
```

Requires `git` on your PATH.

## Quick start

```sh
cd your-monorepo
ripple init            # scaffold a commented ripple.toml
$EDITOR ripple.toml    # declare your modules
ripple validate        # check the config and graph
ripple changed         # what does your current work affect?
```

## Configuration

`ripple.toml` lives at the repository root:

```toml
base = "main"                          # base branch for `ripple changed` (default: main)
include = ["services/*/ripple.toml"]   # optional: merge module tables from other files

[modules.core]
path = "libs/core"

[modules.api]
path = ["services/api", "proto/api/**"]
deps = ["core"]

[modules.web]
path = "apps/web"
deps = ["api"]
```

- **path** — a string or array. Entries containing glob characters (`*?[{`) are matched as globs; everything else is a directory prefix. Paths are relative to the repository root.
- **deps** — names of other modules this module depends on. A change in a dependency marks this module as indirectly affected.
- **include** — glob patterns for fragment files containing additional `[modules.*]` tables. Paths inside a fragment are relative to the fragment's directory, so a `services/api/ripple.toml` can declare `path = "."`. Module names must be unique across all files.

## Commands

Every command documents itself: `ripple <command> --help` includes worked examples.

### `ripple changed`

Lists affected modules. The default diff is your working tree — including uncommitted and untracked files — against the merge-base of HEAD and the base branch, which matches what a pull request from your branch would contain.

```sh
ripple changed                   # working tree vs merge-base with main
ripple changed --base develop    # different base branch
ripple changed main...HEAD       # merge-base diff between two refs (CI)
ripple changed HEAD~3..HEAD      # literal diff between two refs
ripple changed --staged          # staged changes only (pre-commit)
ripple changed --direct-only     # skip transitive dependents
ripple changed --strict          # fail if any changed file has no owning module
```

Output formats (`--format`): `table` (default on a TTY), `plain` (default when piped, one module per line), `json`, and `github` (a GitHub Actions matrix include list). Changed files that no module owns are reported on stderr.

### `ripple validate`

Checks the config: parse errors, duplicate module names, unknown dependencies (with nearest-name suggestions), dependency cycles, missing paths, and overlapping path warnings. Non-zero exit on any error.

### `ripple graph`

```sh
ripple graph                      # list all modules and their deps
ripple graph web --deps           # everything web depends on, as a tree
ripple graph core --dependents    # everything a core change ripples to
ripple graph --format mermaid     # whole graph for a markdown doc
ripple graph --format dot | dot -Tsvg > graph.svg
```

### `ripple explain`

Shows why a module is, or is not, affected: the exact matched files for a direct change, or the dependency chain for an indirect one.

```sh
ripple explain web
```

### `ripple init` / `ripple completions`

`init` writes a commented starter config. `completions <shell>` generates shell completions for bash, zsh, fish, and others.

## CI recipes

Run only the affected modules' jobs in GitHub Actions:

```yaml
jobs:
  detect:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.ripple.outputs.matrix }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - id: ripple
        run: echo "matrix=$(ripple changed origin/main...HEAD --format github)" >> "$GITHUB_OUTPUT"

  test:
    needs: detect
    if: needs.detect.outputs.matrix != '{"include":[]}'
    strategy:
      matrix: ${{ fromJson(needs.detect.outputs.matrix) }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: ./ci/test-module.sh ${{ matrix.module }}
```

Keep the module map honest by validating it and rejecting unowned files:

```yaml
      - run: ripple validate
      - run: ripple changed origin/main...HEAD --strict > /dev/null
```

## Exit codes

`0` success (including "no modules affected"), `1` error (bad config, git failure, `--strict` violation), `2` usage error.

## Development

```sh
just check    # fmt --check, clippy -D warnings, tests
just test
just build
```
