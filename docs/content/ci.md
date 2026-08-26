+++
title = "CI"
description = "Run only the affected modules' jobs in CI"
+++

Ripple's `--format github` emits a GitHub Actions matrix include list, so a single detect job can fan out to exactly the modules a pull request affects.

## GitHub Actions matrix

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

Two details matter here:

- `fetch-depth: 0` — ripple needs enough history to find the merge-base between the PR branch and `origin/main`.
- `origin/main...HEAD` — the three-dot merge-base diff matches what the pull request actually contains, independent of how far `main` has moved since branching.

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

`--format json` carries the full report (status, matched files, dependency chains) for anything richer.

## Pre-commit

`--staged` scopes the diff to what's about to be committed:

```sh
ripple changed --staged --strict
```
