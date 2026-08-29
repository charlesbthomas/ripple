set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
run_sh="$repo_root/changed/run.sh"
ripple_bin="$repo_root/target/debug"
[ -x "$ripple_bin/ripple" ] || { echo "build ripple first: cargo build"; exit 1; }
export PATH="$ripple_bin:$PATH"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

origin="$work/origin"
clone="$work/clone"
mkdir -p "$origin"
git -C "$origin" init -q -b main
git -C "$origin" config user.email test@test.invalid
git -C "$origin" config user.name test
mkdir -p "$origin/libs/core" "$origin/services/api" "$origin/docs"
cat > "$origin/ripple.toml" <<'EOF'
[modules.core]
path = "libs/core"

[modules.api]
path = "services/api"
deps = ["core"]

[modules.docs]
path = "docs"
EOF
echo core > "$origin/libs/core/lib.rs"
echo api > "$origin/services/api/main.rs"
echo docs > "$origin/docs/readme.md"
git -C "$origin" add -A
git -C "$origin" commit -qm init

git clone -q "$origin" "$clone"
git -C "$clone" config user.email test@test.invalid
git -C "$clone" config user.name test
git -C "$clone" checkout -qb feature
echo "core changed" > "$clone/libs/core/lib.rs"
git -C "$clone" commit -qam "core change"
cd "$clone"

fail() { echo "FAIL: $1"; exit 1; }

expect_output() {
  local out="$1" key="$2" want="$3"
  local got
  got="$(grep "^${key}=" "$out" | cut -d= -f2-)"
  [ "$got" = "$want" ] || fail "$key: got '$got', want '$want'"
}

echo "case: pull_request determinate"
out="$(mktemp)"
GITHUB_EVENT_NAME=pull_request GITHUB_BASE_REF=main GITHUB_OUTPUT="$out" bash "$run_sh"
expect_output "$out" modules '["api","core"]'
expect_output "$out" matrix '{"include":[{"module":"api"},{"module":"core"}]}'
expect_output "$out" any-changed true

echo "case: pull_request with filter, no match"
out="$(mktemp)"
GITHUB_EVENT_NAME=pull_request GITHUB_BASE_REF=main GITHUB_OUTPUT="$out" RIPPLE_FILTER=docs bash "$run_sh"
expect_output "$out" modules '[]'
expect_output "$out" any-changed false

echo "case: push with valid before SHA"
out="$(mktemp)"
GITHUB_EVENT_NAME=push EVENT_BEFORE="$(git rev-parse HEAD~1)" GITHUB_OUTPUT="$out" bash "$run_sh"
expect_output "$out" modules '["api","core"]'

echo "case: push new branch falls back to all"
out="$(mktemp)"
GITHUB_EVENT_NAME=push EVENT_BEFORE="0000000000000000000000000000000000000000" GITHUB_OUTPUT="$out" bash "$run_sh"
expect_output "$out" modules '["api","core","docs"]'
expect_output "$out" any-changed true

echo "case: workflow_dispatch falls back to all, filtered"
out="$(mktemp)"
GITHUB_EVENT_NAME=workflow_dispatch GITHUB_OUTPUT="$out" RIPPLE_FILTER="docs,api" bash "$run_sh"
expect_output "$out" modules '["api","docs"]'

echo "case: fallback=error exits nonzero"
out="$(mktemp)"
if GITHUB_EVENT_NAME=workflow_dispatch GITHUB_OUTPUT="$out" RIPPLE_FALLBACK=error bash "$run_sh" 2>/dev/null; then
  fail "expected nonzero exit for fallback=error"
fi

echo "ok: all changed/run.sh cases passed"
