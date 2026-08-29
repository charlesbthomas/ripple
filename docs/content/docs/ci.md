+++
title = "CI"
description = "Run only the affected modules' jobs in CI"
weight = 4
aliases = ["/ci/"]
+++

Ripple ships two composite actions: a setup action that installs the binary, and a `changed` action that detects affected modules and exposes them as job outputs — no shell scripting required.

## The changed action

`charlesbthomas/ripple/changed` installs ripple, derives the diff range from the event that triggered the workflow, runs the detection, and publishes the result as outputs:

{% raw %}
```yaml
jobs:
  detect:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.ripple.outputs.matrix }}
      any-changed: ${{ steps.ripple.outputs.any-changed }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: charlesbthomas/ripple/changed@v0.2.0
        id: ripple

  test:
    needs: detect
    if: needs.detect.outputs.any-changed == 'true'
    strategy:
      matrix: ${{ fromJson(needs.detect.outputs.matrix) }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: ./ci/test-module.sh ${{ matrix.module }}
```
{% endraw %}

### Inputs

| Input | Default | Meaning |
|-------|---------|---------|
| `version` | `latest` | ripple release to install (`latest`, `0.2.0`, or `v0.2.0`) |
| `filter` | *(empty)* | Comma-separated module names; restrict the report to these modules |
| `fallback` | `all` | What to do when no diff range can be determined: `all` treats every module as changed, `error` fails the step |

### Outputs

| Output | Example | Meaning |
|--------|---------|---------|
| `modules` | `["api","core"]` | JSON array of affected module names |
| `matrix` | `{"include":[{"module":"api"}]}` | GitHub Actions matrix include list |
| `any-changed` | `true` | Whether any module is affected |

### Range derivation

The action picks the diff range from the workflow trigger, so the same step works everywhere:

- `pull_request` / `pull_request_target` — merge-base diff against the PR base branch
- `push` — `event.before..HEAD`, guarding against new branches and force pushes
- `merge_group` — diff against the merge queue base
- anything else (e.g. `workflow_dispatch`) — no range; the `fallback` input decides

When a range cannot be determined and `fallback` is `all`, every configured module is reported as changed — CI runs everything rather than silently skipping work.

Checkout with `fetch-depth: 0` so the merge-base and `event.before` commits are available; a shallow checkout degrades to the fallback behavior.

Hosted runners include `jq`, which the action uses; self-hosted runners need it on `PATH`. Linux and macOS only.

### Gating a job on specific modules

`filter` answers "did anything I care about change?" without a matrix:

{% raw %}
```yaml
  detect:
    runs-on: ubuntu-latest
    outputs:
      run: ${{ steps.ripple.outputs.any-changed }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: charlesbthomas/ripple/changed@v0.2.0
        id: ripple
        with:
          filter: core,api,worker

  test:
    needs: detect
    if: needs.detect.outputs.run == 'true'
    ...
```
{% endraw %}

Unknown module names in `filter` fail the step, so a typo cannot silently gate a job off forever.

## Installing only

The repository root doubles as a setup action that downloads a release binary, verifies its checksum, and puts `ripple` on `PATH` — use it when you want to run ripple commands yourself:

```yaml
      - uses: charlesbthomas/ripple@v0.2.0
        with:
          version: latest    # or a specific release, e.g. "0.2.0"
      - run: ripple --version
```

The `version` input accepts `latest` (default), `0.2.0`, or `v0.2.0`. The actions ship with every release tag from `v0.1.1` onward, so the action ref and the installed version can be pinned independently.

## Guardrails

Keep the module map honest by validating it and rejecting unowned files on every PR:

```yaml
      - run: ripple validate
      - run: ripple changed origin/main...HEAD --strict > /dev/null
```

`validate` catches config drift (cycles, unknown deps, missing paths); `--strict` fails the build when a changed file has no owning module, so new code can't silently escape the map.

## Other CI systems

`--format plain` prints one affected module per line, which shells into any CI:

```sh
for module in $(ripple changed origin/main...HEAD --format plain); do
  ./ci/test-module.sh "$module"
done
```

`--filter` gates on specific modules without grep:

```sh
if [ -n "$(ripple changed origin/main...HEAD --filter core,api --format plain)" ]; then
  ./ci/run-backend-tests.sh
fi
```

`--format json` carries the full report (status, matched files, dependency chains) for anything richer, and `ripple list --format json` enumerates every module when you need a "run everything" fallback.

## Pre-commit

`--staged` scopes the diff to what's about to be committed:

```sh
ripple changed --staged --strict
```
