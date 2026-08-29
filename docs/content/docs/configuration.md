+++
title = "Configuration"
description = "The ripple.toml reference"
weight = 2
aliases = ["/configuration/"]
+++

`ripple.toml` lives at the repository root. Ripple finds it from any subdirectory, so commands work wherever you are in the repo.

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

## Top-level keys

### `base`

The base branch `ripple changed` diffs against when you don't pass an explicit range. Defaults to `main`. Override per invocation with `--base <ref>`.

### `include`

Glob patterns for fragment files containing additional `[modules.*]` tables. This lets each service own its module declaration:

```toml
include = ["services/*/ripple.toml"]
```

Paths inside a fragment are relative to the fragment's directory, so a `services/api/ripple.toml` can simply declare:

```toml
[modules.api]
path = "."
deps = ["core"]
```

Module names must be unique across all files — a duplicate is a validation error.

## Module keys

### `path`

A string or an array of strings, relative to the repository root (or the fragment's directory).

Entries containing glob characters (`*`, `?`, `[`, `{`) are matched as globs; everything else is treated as a directory prefix:

```toml
[modules.api]
path = ["services/api", "proto/api/**"]
```

Here `services/api` owns every file under that directory, while `proto/api/**` is a glob match.

### `deps`

Names of other modules this module depends on. A change in a dependency (direct or transitive) marks this module as **indirectly** affected:

```toml
[modules.web]
path = "apps/web"
deps = ["api"]
```

## Keeping the config honest

Two guardrails keep the module map accurate as the repository grows:

- `ripple validate` — catches parse errors, duplicate names, unknown deps (with nearest-name suggestions), dependency cycles, missing paths, and overlapping path warnings.
- `ripple changed --strict` — fails if any changed file has no owning module, so new directories can't silently escape the map.
