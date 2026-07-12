#!/usr/bin/env bash
# statusline.sh — emit a one-line bd-progress summary for Claude Code.
#
# Reads bd state via `bd stats --json` and `bd ready --json`. bd is the
# canonical work tracker for this repo (see CLAUDE.md "Beads Issue
# Tracker"); this script gives Claude Code a glance at it.
#
# Self-locating: walks up from $PWD until it finds a .beads/ directory.
# Outputs nothing (graceful no-op) when run outside a bd-tracked repo,
# when bd or jq isn't on PATH, or when bd commands fail — so it's safe
# to wire globally.
#
# Output shape:
#   📋 LLM-Wiki · 38/43 closed · next: LLM-Wiki-yh0 P0 Revoke stale GITHUB_TOKEN PAT
#   📋 LLM-Wiki · 43/43 closed · all clear
#
# Implementation note: the script reads bd state read-only. Recent
# additions/changes are reflected on the next invocation (the harness
# re-runs this for each status refresh).

set -euo pipefail

# Self-locate the bd-tracked repo root. Stop if dirname stops making progress
# (it is idempotent at a root): on some path forms that fixpoint is not "/", and
# looping there would hang every status refresh.
dir="${PWD}"
while [ ! -d "$dir/.beads" ]; do
    parent="$(dirname "$dir")"
    [ "$parent" != "$dir" ] || break
    dir="$parent"
done

# Graceful no-op when prerequisites are missing.
[ -d "$dir/.beads" ] || exit 0
command -v bd >/dev/null 2>&1 || exit 0
command -v jq >/dev/null 2>&1 || exit 0

repo="$(basename "$dir")"

# Pull stats. On failure bd exits non-zero, leaves stdout empty, and describes
# the cause as a JSON object on *stderr* -- that is how the remote-migrate gate
# reports itself, and it blocks every read until an operator resolves it. This
# used to fall through to a silent `exit 0`, which renders as an empty status
# line: indistinguishable from a broken harness, and it hides the very state you
# need to act on. So keep stderr and report it. Past this point we are in a bd
# repo with bd installed, so always print something.
errfile="$(mktemp)"
trap 'rm -f "$errfile"' EXIT
stats="$(cd "$dir" && bd stats --json 2>"$errfile")" || stats=""

total="$(printf  '%s' "$stats" | jq -r '.summary.total_issues  // empty' 2>/dev/null)" || total=""
closed="$(printf '%s' "$stats" | jq -r '.summary.closed_issues // empty' 2>/dev/null)" || closed=""

if [ -z "$total" ] || [ -z "$closed" ]; then
    # The migrate-or-adopt choice is the operator's (migrating a second clone
    # forks the schema unrecoverably), so name the versions and both exits
    # rather than implying a single fix.
    cur="$(jq    -r '.remote_migrate_gate.current_version // empty' "$errfile" 2>/dev/null)" || cur=""
    latest="$(jq -r '.remote_migrate_gate.latest_version  // empty' "$errfile" 2>/dev/null)" || latest=""
    if [ -n "$cur" ] && [ -n "$latest" ]; then
        printf '📋 %s · ⚠ bd schema gate v%s→v%s · bd migrate (designated clone) | bd bootstrap (adopt)\n' \
            "$repo" "$cur" "$latest"
    else
        printf '📋 %s · ⚠ bd unavailable\n' "$repo"
    fi
    exit 0
fi

# bd ready returns ready issues sorted by priority ascending (P0 first).
ready="$(cd "$dir" && bd ready --json 2>/dev/null)" || ready="[]"
next_id="$(printf    '%s' "$ready" | jq -r '.[0].id       // empty')"
next_pri="$(printf   '%s' "$ready" | jq -r '.[0].priority // empty')"
next_title="$(printf '%s' "$ready" | jq -r '.[0].title    // empty')"

if [ -z "$next_id" ]; then
    printf '📋 %s · %s/%s closed · all clear\n' "$repo" "$closed" "$total"
else
    # Truncate the title to keep the status line tight.
    if [ ${#next_title} -gt 50 ]; then
        next_title="${next_title:0:47}..."
    fi
    printf '📋 %s · %s/%s closed · next: %s P%s %s\n' \
        "$repo" "$closed" "$total" "$next_id" "$next_pri" "$next_title"
fi
