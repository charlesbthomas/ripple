+++
title = "ripple"
description = "Monorepo change detection: which modules changed for a given git diff"

[extra]
home_eyebrow = "Monorepo change detection"
home_primary_action_label = "Get started"
home_primary_action_path = "/docs/getting-started/"
home_secondary_action_label = "View docs"
home_secondary_action_path = "/docs/"
home_features = [
  { kicker = "Detect", title = "Direct and indirect changes", description = "Reports the modules whose files changed and every module that transitively depends on them — the ripple effect." },
  { kicker = "Declare", title = "One config file", description = "Describe your module map once in ripple.toml, then use the same answer locally and in CI." },
  { kicker = "CI", title = "Run only what changed", description = "Composite GitHub Actions expose changed modules as job outputs — no shell scripting required." },
]
+++

Ripple answers one question: given a git diff, **which modules of your monorepo changed?** It reports both the modules whose files changed and every module that transitively depends on them — the ripple effect.

```
$ ripple changed
MODULE  STATUS    VIA
api     indirect  core
core    direct
web     indirect  api -> core

3 modules affected (1 direct, 2 indirect)
```

Declare your dependency graph once in `ripple.toml`, then use one command locally and in CI to answer "what does this change affect?"

## Install

```sh
brew install charlesbthomas/tap/ripple
```

Requires `git` on your PATH. See [Getting Started](@/docs/getting-started.md) for other install methods.

## Quick start

```sh
cd your-monorepo
ripple init            # scaffold a commented ripple.toml
$EDITOR ripple.toml    # declare your modules
ripple validate        # check the config and graph
ripple changed         # what does your current work affect?
```

## Learn more

- [Getting Started](@/docs/getting-started.md) — install ripple and wire up your first module map
- [Configuration](@/docs/configuration.md) — the `ripple.toml` reference
- [Commands](@/docs/commands.md) — `changed`, `validate`, `graph`, `explain`, `init`, `completions`
- [CI](@/docs/ci.md) — run only affected modules' jobs in GitHub Actions
