+++
title = "Getting Started"
description = "Install ripple and wire up your first module map"
+++

## Install

Ripple is a single binary with one external requirement: `git` on your PATH.

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

## Scaffold a config

From your monorepo root:

```sh
ripple init
```

This writes a `ripple.toml` with commented examples covering modules, paths, globs, dependencies, and includes. It refuses to overwrite an existing file.

## Declare your modules

Edit `ripple.toml` to describe your repository. A module is a name, the paths it owns, and the modules it depends on:

```toml
[modules.core]
path = "libs/core"

[modules.api]
path = "services/api"
deps = ["core"]

[modules.web]
path = "apps/web"
deps = ["api"]
```

## Validate

```sh
ripple validate
```

This catches parse errors, duplicate module names, unknown dependencies (with nearest-name suggestions), dependency cycles, and paths that do not exist on disk. It exits non-zero on any error, so it also slots into CI or a pre-commit hook.

## See what your work affects

```sh
ripple changed
```

By default this diffs your working tree — including uncommitted and untracked files — against the merge-base of HEAD and the base branch. That matches what a pull request from your branch would contain.

```
MODULE  STATUS    VIA
api     indirect  core
core    direct
web     indirect  api -> core

3 modules affected (1 direct, 2 indirect)
```

A module is **direct** when its own files changed, and **indirect** when something it depends on changed. `ripple explain <module>` shows the exact files or dependency chain behind either verdict.

## Next steps

- The full `ripple.toml` reference: [Configuration](@/configuration.md)
- Every command and flag: [Commands](@/commands.md)
- Run only affected jobs in GitHub Actions: [CI](@/ci.md)
