#!/usr/bin/env bash
# statusline.sh — one-line Claude Code status line for this repo:
#
#   LEPTOS-DAISYUI-RS | 1 open / 498 total | next: 9ike | 7d: 80% left | ctx: 72% left | model: opus | effort: High
#
# - Beads use the SAME source and rule as office-builder's shared ~/.claude/statusline.py
#   (owner 2026-10-30), so every session shows one count:
#   source = fork/main's .beads/issues.jsonl (`git show`), refreshed by a background
#   `git fetch fork main` at most every 10 minutes that never blocks the line.
#   This repo's `origin` is the upstream noshishiRust repo, which carries no beads;
#   the fork (davidcforbes) is where the export is pushed. Ported from 4iiz-etl
#   (scripts/statusline.sh 71d23a2d) with only the label and the remote changed;
#   the working tree's export is the fallback when fork/main has none.
#   open = status open, in_progress or blocked, gates left out (deferred beads have
#   their own status, so they are out); total = every issue incl. closed.
#   next = the best ready bead: status open, not an epic or gate, not deferred to a
#   later time, no open `blocks` dependency; lowest priority number, then oldest.
#   Short id only (after the last "-"); "n/a" when none. Cached 60 s.
#   The beads segment is omitted outside a bd-tracked repo.
# - 7d / ctx / model / effort come from the status line's stdin JSON (rate_limits.seven_day,
#   context_window, the family word of model.display_name in lower case, effort.level
#   capitalised; owner 2026-10-31: no 5h window). Any field the JSON does not carry
#   (e.g. no subscription rate limits, or no API response yet) prints "n/a".
# - Also keeps the user-level onwatch capture alive: a project statusLine
#   replaces the user-level one, which is what fed ~/.onwatch/data.
#
# Reads state read-only. Never fails the harness: always exits 0.

set -u

input="$(cat)"

# Preserve the user-level onwatch capture (atomic write, errors ignored).
(
    d="$HOME/.onwatch/data"
    mkdir -p "$d" 2>/dev/null
    t="$d/.sl-$$"
    printf '%s' "$input" >"$t" && mv -f "$t" "$d/anthropic-statusline.json" 2>/dev/null || rm -f "$t" 2>/dev/null
) 2>/dev/null || true

if ! command -v jq >/dev/null 2>&1; then
    printf 'statusline: jq not found\n'
    exit 0
fi

# --- Limits and context, from the stdin JSON -------------------------------
week=""; ctx=""; model=""; effort=""
read -r week ctx model effort < <(printf '%s' "$input" | jq -r '
    def left(x): if x == null then "n/a"
                 else ((100 - x) | if . < 0 then 0 else . end | round | tostring) + "%" end;
    [ left(.rate_limits.seven_day.used_percentage),
      (if .context_window.remaining_percentage != null
         then (.context_window.remaining_percentage | round | tostring) + "%"
         else left(.context_window.used_percentage) end),
      ((.model.display_name // .model.id // "n/a") | split(" ")[0] | ascii_downcase
         | if startswith("claude-") then split("-")[1] else . end),
      ((.effort.level // "n/a") | tostring
         | if . == "n/a" then . else (.[0:1] | ascii_upcase) + .[1:] end)
    ] | join(" ")' 2>/dev/null) || true
week="${week:-n/a}"; ctx="${ctx:-n/a}"; model="${model:-n/a}"; effort="${effort:-n/a}"

# --- Beads, from the nearest .beads/ ---------------------------------------
dir="${PWD}"
while [ "$dir" != "/" ] && [ "$dir" != "." ] && [ ! -d "$dir/.beads" ]; do
    dir="$(dirname "$dir")"
done

beads=""
if [ -d "$dir/.beads" ]; then
    repo="$(basename "$dir")"
    cache_dir="$HOME/.claude/statusline-cache"
    cache_file="$cache_dir/beads-$repo.txt"
    counts=""

    # Fresh cache (under 60 s)? Format "<open> <total> <next>"; an older
    # two-field cache is ignored so it gets rebuilt with the next id.
    if [ -f "$cache_file" ]; then
        now="$(date +%s)"
        mtime="$(stat -c %Y "$cache_file" 2>/dev/null || echo 0)"
        if [ $((now - mtime)) -lt 60 ]; then
            counts="$(cat "$cache_file" 2>/dev/null || true)"
            case "$counts" in *" "*" "*) ;; *) counts="" ;; esac
        fi
    fi

    # Refresh fork/main in the background, at most every 10 minutes, never blocking.
    stamp="$cache_dir/fetch-$repo.stamp"
    now="$(date +%s)"
    smtime="$(stat -c %Y "$stamp" 2>/dev/null || echo 0)"
    if [ $((now - smtime)) -ge 600 ]; then
        mkdir -p "$cache_dir" 2>/dev/null || true
        touch "$stamp" 2>/dev/null || true
        ( cd "$dir" && GIT_TERMINAL_PROMPT=0 git fetch -q fork main ) </dev/null >/dev/null 2>&1 &
    fi

    if [ -z "$counts" ]; then
        export_json="$(cd "$dir" && git show fork/main:.beads/issues.jsonl 2>/dev/null)" || export_json=""
        if [ -z "$export_json" ] && [ -f "$dir/.beads/issues.jsonl" ]; then
            export_json="$(cat "$dir/.beads/issues.jsonl" 2>/dev/null)" || export_json=""
        fi
        counts="$(printf '%s' "$export_json" | jq -rs --arg now "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
            [.[] | select(.status != null)] as $all
            | if ($all | length) == 0 then empty else
              ($all | map({key: .id, value: .status}) | from_entries) as $st
              | [$all[] | select((.status == "open" or .status == "in_progress" or .status == "blocked")
                                 and .issue_type != "gate")] as $open
              | [$all[]
                 | select(.status == "open" and .issue_type != "epic" and .issue_type != "gate"
                          and (((.defer_until // "") == "") or (.defer_until <= $now))
                          and ([(.dependencies // [])[] | select(.type == "blocks")
                                | ($st[.depends_on_id] // "closed") | select(. != "closed")] | length) == 0)]
                | sort_by(.priority, .created_at) as $ready
              | ($ready[0].id // "" | split("-") | last) as $next
              | "\($open | length) \($all | length) \(if ($next // "") == "" then "n/a" else $next end)"
              end' 2>/dev/null)" || counts=""
        if [ -n "$counts" ]; then
            mkdir -p "$cache_dir" 2>/dev/null || true
            printf '%s' "$counts" >"$cache_file.$$" 2>/dev/null \
                && mv -f "$cache_file.$$" "$cache_file" 2>/dev/null || rm -f "$cache_file.$$" 2>/dev/null || true
        fi
    fi

    if [ -n "$counts" ]; then
        open_n="${counts%% *}"; rest="${counts#* }"
        total_n="${rest%% *}"; next_n="${rest#* }"
        beads="${open_n} open / ${total_n} total | next: ${next_n:-n/a}"
    else
        beads="beads n/a | next: n/a"
    fi
fi

out=""
[ -n "$beads" ] && out="LEPTOS-DAISYUI-RS | $beads | "
printf '%s7d: %s left | ctx: %s left | model: %s | effort: %s\n' "$out" "$week" "$ctx" "$model" "$effort"
exit 0
