+++
title = "ripple"
description = "Monorepo change detection: which modules changed for a given git diff"
template = "index.html"
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

Requires `git` on your PATH. See [Getting Started](@/getting-started.md) for other install methods.

## Quick start

```sh
cd your-monorepo
ripple init            # scaffold a commented ripple.toml
$EDITOR ripple.toml    # declare your modules
ripple validate        # check the config and graph
ripple changed         # what does your current work affect?
```

## Learn more

- [Getting Started](@/getting-started.md) — install ripple and wire up your first module map
- [Configuration](@/configuration.md) — the `ripple.toml` reference
- [Commands](@/commands.md) — `changed`, `validate`, `graph`, `explain`, `init`, `completions`
- [CI](@/ci.md) — run only affected modules' jobs in GitHub Actions
