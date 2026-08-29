set -euo pipefail

range=""
indeterminate_reason=""

derive_range() {
  case "${GITHUB_EVENT_NAME:-}" in
    pull_request|pull_request_target)
      git fetch --no-tags --force origin \
        "+refs/heads/${GITHUB_BASE_REF}:refs/remotes/origin/${GITHUB_BASE_REF}" 2>/dev/null || true
      if git merge-base "origin/${GITHUB_BASE_REF}" HEAD >/dev/null 2>&1; then
        range="origin/${GITHUB_BASE_REF}...HEAD"
      else
        indeterminate_reason="no merge-base with origin/${GITHUB_BASE_REF} (shallow checkout? use fetch-depth: 0)"
      fi
      ;;
    push)
      if [ -z "${EVENT_BEFORE:-}" ] || [ "$EVENT_BEFORE" = "0000000000000000000000000000000000000000" ]; then
        indeterminate_reason="push with no before SHA (new branch or tag)"
      elif ! git cat-file -e "${EVENT_BEFORE}^{commit}" 2>/dev/null; then
        indeterminate_reason="before SHA ${EVENT_BEFORE} not in history (force push or shallow checkout)"
      else
        range="${EVENT_BEFORE}..HEAD"
      fi
      ;;
    merge_group)
      if [ -n "${MERGE_GROUP_BASE_SHA:-}" ] && git cat-file -e "${MERGE_GROUP_BASE_SHA}^{commit}" 2>/dev/null; then
        range="${MERGE_GROUP_BASE_SHA}..HEAD"
      else
        indeterminate_reason="merge_group base SHA unavailable"
      fi
      ;;
    *)
      indeterminate_reason="event ${GITHUB_EVENT_NAME:-unknown} has no diff range"
      ;;
  esac
}
derive_range

if [ -n "$indeterminate_reason" ]; then
  if [ "${RIPPLE_FALLBACK:-all}" = "error" ]; then
    echo "::error::cannot determine diff range: ${indeterminate_reason}"
    exit 1
  fi
  echo "::warning::cannot determine diff range (${indeterminate_reason}); treating all modules as changed"
  modules="$(ripple list --format json)"
  if [ -n "${RIPPLE_FILTER:-}" ]; then
    modules="$(jq -c --arg f "$RIPPLE_FILTER" '[.[] | select(. as $m | ($f | split(",")) | index($m))]' <<<"$modules")"
  fi
else
  echo "detecting changed modules over ${range}"
  args=(changed "$range" --format json)
  if [ -n "${RIPPLE_FILTER:-}" ]; then
    args+=(--filter "$RIPPLE_FILTER")
  fi
  report="$(ripple "${args[@]}")"
  modules="$(jq -c '[.modules[].name]' <<<"$report")"
fi

matrix="$(jq -c '{include: map({module: .})}' <<<"$modules")"
if [ "$modules" = "[]" ]; then
  any_changed=false
else
  any_changed=true
fi

echo "affected modules: $modules"
{
  echo "modules=$modules"
  echo "matrix=$matrix"
  echo "any-changed=$any_changed"
} >> "$GITHUB_OUTPUT"
