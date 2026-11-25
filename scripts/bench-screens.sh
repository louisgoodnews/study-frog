#!/usr/bin/env bash
# Scripted visit of the rehearsal card, the reports screen and the console, timed (EPIC-067, TASK-0284, STORY-0204).
#
# Usage: scripts/bench-screens.sh <vault-dir> <out-dir> [passes=3]
#   <vault-dir>  an opened-able vault, e.g. one made by scripts/gen-vault-10k.sh
#   <out-dir>    results folder: timings.csv and shots/ (screenshots, not timed)
#   [passes=3]   each pass visits the three screens in turn; the medians are over the passes
#   Environment: STUDYFROG_BIN (default: studyfrog), STUDYFROG_SOCKET (passed to ctl as --socket)
#
# NEEDS A DISPLAY AND A RUNNING WINDOW. Start StudyFrog with its window first (not --headless); the script
# only talks to it through `studyfrog ctl`. The screenshots need the window too; their failure is not an error.
# The timings are wall-clock round trips of control commands, from `date +%s%N` before to after each one.
# An answer that waits for the app's frame includes that frame, so the numbers are a visit's cost as seen by
# the person driving it. They are NOT frame time: the per-frame CPU summary (median, p95, worst frame) is the
# opt-in capture of TASK-0283, which this script does not switch on. GPU, compositor and input latency are
# not measured here either.
#
# NEEDS THE OWNER'S MACHINE for the numbers that count: a container with a virtual display runs the script
# but is not a reference machine (see BENCHMARKING.md). A screen that cannot be opened or driven here is not
# measured; the script names it at the end ("not measured") and records why in timings.csv.
#
# timings.csv columns: pass,screen,step,elapsed_ms,ok,note. Step "visit" is the whole visit of one screen,
# from its first command to its last (including the settle that waits for background work).
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "usage: $0 <vault-dir> <out-dir> [passes=3]" >&2
    exit 2
fi
vault="$1"
out="$2"
passes="${3:-3}"
case "$passes" in '' | *[!0-9]*) echo "passes must be a positive integer" >&2; exit 2 ;; esac
[ "$passes" -ge 1 ] || { echo "passes must be a positive integer" >&2; exit 2; }
[ -d "$vault" ] || { echo "error: no vault folder at $vault" >&2; exit 2; }

bin="${STUDYFROG_BIN:-studyfrog}"
command -v "$bin" >/dev/null 2>&1 || { echo "error: '$bin' not found; set STUDYFROG_BIN" >&2; exit 2; }

mkdir -p "$out/shots"
vault="$(cd "$vault" && pwd)"
out="$(cd "$out" && pwd)"
csv="$out/timings.csv"
echo "pass,screen,step,elapsed_ms,ok,note" >"$csv"

# ctl WORDS...: one control request; prints its answer. The socket flag is given only when set.
ctl() {
    if [ -n "${STUDYFROG_SOCKET:-}" ]; then
        "$bin" ctl --socket "$STUDYFROG_SOCKET" "$@" </dev/null
    else
        "$bin" ctl "$@" </dev/null
    fi
}

# Milliseconds between two nanosecond stamps (integers, so no precision is lost before the division).
ms_between() { awk -v d="$(($2 - $1))" 'BEGIN { printf "%.3f", d / 1000000 }'; }

# step PASS SCREEN NAME WORDS...: runs one control command, timed, and appends its row. Returns 1 when the
# app refused it (its answer has no "ok":true).
step() {
    local pass="$1" screen="$2" name="$3" t0 t1 answer ok=0 note=""
    shift 3
    t0="$(date +%s%N)"
    answer="$(ctl "$@" 2>&1)" || true
    t1="$(date +%s%N)"
    case "$answer" in *'"ok":true'*) ok=1 ;; esac
    if [ "$ok" = 0 ]; then
        note="$(printf '%s' "$answer" | head -c 160 | tr ',\n"' '   ')"
    fi
    printf '%s,%s,%s,%s,%s,%s\n' "$pass" "$screen" "$name" "$(ms_between "$t0" "$t1")" "$ok" "$note" >>"$csv"
    [ "$ok" = 1 ]
}

# visit PASS SCREEN: reads "name ctl-words..." lines on stdin and runs them in order, stopping at the first
# refusal. Writes the visit row (step "visit") with the wall-clock time of the whole visit.
visit() {
    local pass="$1" screen="$2" t0 t1 ok=1 name words
    t0="$(date +%s%N)"
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        name="${line%% *}"
        words="${line#* }"
        # shellcheck disable=SC2086
        step "$pass" "$screen" "$name" $words || { ok=0; break; }
    done
    t1="$(date +%s%N)"
    printf '%s,%s,%s,%s,%s,%s\n' "$pass" "$screen" visit "$(ms_between "$t0" "$t1")" "$ok" "" >>"$csv"
    [ "$ok" = 1 ]
}

# The three visits. Each is a fixed interaction script, so passes are comparable.
visit_rehearsal() {
    visit "$1" rehearsal <<'STEPS' || return 1
navigate go rehearse
settle settle
setup rehearse setup --preset spaced --mode flip --limit 50
start rehearse start
flip rehearse flip
rate rehearse rate good
stop rehearse stop
STEPS
}

visit_reports() {
    visit "$1" reports <<'STEPS' || return 1
navigate go reports
settle settle
performance reports performance
familiarity reports familiarity
STEPS
}

visit_console() {
    visit "$1" console <<'STEPS' || return 1
navigate go console
settle settle
logs logs --tail 50
STEPS
}

# The state check confirms a window is answering; then the vault is opened once, before any timed pass.
step 0 setup state state >/dev/null 2>&1 || {
    echo "error: no running StudyFrog window answered 'ctl state' (see $csv); start the app with its window" >&2
    exit 1
}
step 0 setup vault_open vault open "$vault" || { echo "error: vault open refused (see $csv)" >&2; exit 1; }

failed_screens=""
pass=0
while [ "$pass" -lt "$passes" ]; do
    pass=$((pass + 1))
    for screen in rehearsal reports console; do
        if ! "visit_$screen" "$pass"; then
            failed_screens="$failed_screens $screen"
        fi
        # A screenshot after the timed visit (not timed); failure is ignored (it needs the window).
        ctl screenshot "$out/shots/pass$pass-$screen.png" >/dev/null 2>&1 || true
    done
done

# Summary: visits that passed per screen and the median of their visit times.
echo "results: $csv"
for screen in rehearsal reports console; do
    ok_visits="$(awk -F, -v s="$screen" '$2 == s && $3 == "visit" && $5 == 1 { print $4 }' "$csv" | sort -g)"
    if [ -n "$ok_visits" ]; then
        median="$(printf '%s\n' "$ok_visits" | awk '{ v[NR] = $1 } END { print (NR % 2) ? v[(NR + 1) / 2] : (v[NR / 2] + v[NR / 2 + 1]) / 2 }')"
        n="$(printf '%s\n' "$ok_visits" | wc -l | tr -d ' ')"
        echo "$screen: visits=$n median_visit_ms=$median"
    else
        echo "$screen: not measured (no visit completed; see the note column in $csv)"
    fi
done
if [ -n "$failed_screens" ]; then
    echo "not measured in this environment:$failed_screens (a step was refused; see $csv)"
else
    echo "all three screens measured on every pass"
fi
