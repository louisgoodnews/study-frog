#!/usr/bin/env bash
# Frame time of the rehearsal card, the reports screen and the console, measured in the running window
# (EPIC-067, STORY-0204, TASK-0283 capture, TASK-0284 scripted visit).
#
# Usage: scripts/bench-frame-time.sh <vault-dir> <out-dir> [passes=3]
#   <vault-dir>  a vault to open, e.g. one made by scripts/gen-vault-10k.sh (10,000 notes)
#   <out-dir>    results folder; must not already hold results (give a new folder)
#   [passes=3]   each pass is a separate app run, so each pass starts with a fresh process
#   Environment:
#     STUDYFROG_BIN          the app binary (default: studyfrog)
#     STUDYFROG_SOCKET       passed to `ctl --socket` when set
#     FRAME_TIME_MACHINE     machine label written into the identity (default: unlabelled)
#     FRAME_TIME_CLOSE_WAIT  seconds to wait for each window to close (default: 1800)
#
# Writes into <out-dir>:
#   machine.txt            machine identity (scripts/bench-machine.sh); warnings in machine-warnings.txt
#   frames-<pass>.csv      the app's own capture: columns frame,screen,frame_us (one row per frame drawn)
#   app-<pass>.log         the app's output for the pass
#   steps.csv              one row per control command: pass,screen,step,elapsed_ms,ok,note
#   summary.csv            one row per pass and screen: median, p95, worst (microseconds) with the identity
#   summary.txt            the same, readable, with the statistics rule, coverage and refused steps
#
# Statistics (the app's own rule, frame_times::summary; scripts/bench-frame-time.sh computes it in the
# embedded python helper, so a reader can check any row by hand from frames-<pass>.csv):
#   n = number of frames of that screen in that pass; the frame times are sorted, smallest first.
#   median: the middle value when n is odd; when n is even, the two middle values added and divided by 2,
#           rounded down (integer division).
#   p95:    nearest rank: rank = ceil(n * 95 / 100), counted from 1 in sorted order; p95 is the value at that rank.
#   worst:  the largest value.
#   Every frame drawn while a screen was showing counts, including idle repaints, so the frame count depends on
#   how often egui redraws. A screen with no frames in a pass is reported as not measured.
#
# NEEDS A DISPLAY AND A PERSON AT THE WINDOW (owner's machine for numbers that count). The app runs with a
# window (no headless mode for this capture). The window is closed by hand at the end of each pass: no
# control command closes it, and the capture is written only when the window closes normally (eframe's exit
# hook). If the window is still open after FRAME_TIME_CLOSE_WAIT seconds, the pass is stopped with SIGTERM and
# its frames file is probably missing; the summary says so. A container with a virtual display can run the
# script, but it is not a reference machine (docs/contributing/BENCHMARKING.md).
#
# NEEDS NO DISPLAY: the 10k vault (scripts/gen-vault-10k.sh), the machine identity and the summary (python3,
# standard library only). The out-dir must be new: a folder with results is refused, so a run is never mixed
# with an older one.
#
# Not measured here: GPU time, compositor and display latency, input latency. Only the UI-thread CPU time
# the app sees for each frame is recorded.
#
# The vault has notes only (gen-vault-10k.sh writes no cards). The rehearsal visit therefore has no card to
# rehearse: its setup may succeed on an empty queue, and start, flip, rate and stop are then refused by the app.
# Refused steps are recorded in steps.csv and listed in summary.txt; the visit goes on, so the frames of the
# rehearsal screen are still measured.
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
command -v python3 >/dev/null 2>&1 || { echo "error: python3 not found (needed for the summary)" >&2; exit 2; }
close_wait="${FRAME_TIME_CLOSE_WAIT:-1800}"
case "$close_wait" in '' | *[!0-9]*) echo "FRAME_TIME_CLOSE_WAIT must be a whole number of seconds" >&2; exit 2 ;; esac

here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
vault="$(cd "$vault" && pwd)"
if [ -n "$(ls -A "$out" 2>/dev/null)" ]; then
    echo "error: $out is not empty; give a new folder for a frame-time run" >&2
    exit 1
fi
steps="$out/steps.csv"
echo "pass,screen,step,elapsed_ms,ok,note" >"$steps"

log() { printf '[frame-time] %s\n' "$*"; }

# ctl WORDS...: one control request to the running window; prints its answer.
ctl() {
    if [ -n "${STUDYFROG_SOCKET:-}" ]; then
        "$bin" ctl --socket "$STUDYFROG_SOCKET" "$@" </dev/null
    else
        "$bin" ctl "$@" </dev/null
    fi
}

# step PASS SCREEN NAME WORDS...: one control command, timed, with its row in steps.csv. Always returns 0:
# a refused command is recorded, and the visit goes on.
step() {
    local pass="$1" screen="$2" name="$3" t0 t1 answer ok=0 note=""
    shift 3
    t0="$(date +%s%N)"
    answer="$(ctl "$@" 2>&1)" || true
    t1="$(date +%s%N)"
    case "$answer" in *'"ok":true'*) ok=1 ;; esac
    if [ "$ok" = 0 ]; then
        note="$(printf '%s' "$answer" | head -c 160 | tr ',\n"' '   ')"
        log "pass $pass: $screen $name refused:$note"
    fi
    printf '%s,%s,%s,%s,%s,%s\n' "$pass" "$screen" "$name" \
        "$(awk -v d="$((t1 - t0))" 'BEGIN { printf "%.3f", d / 1000000 }')" "$ok" "$note" >>"$steps"
    return 0
}

# The three visits, in the same order each pass. Each is a fixed list of control commands.
visit_rehearsal() {
    local pass="$1"
    step "$pass" rehearse nav nav rehearse
    step "$pass" rehearse settle settle
    step "$pass" rehearse setup rehearse setup --preset spaced --mode flip --limit 50
    step "$pass" rehearse start rehearse start
    step "$pass" rehearse flip rehearse flip
    step "$pass" rehearse rate rehearse rate good
    step "$pass" rehearse stop rehearse stop
}

visit_reports() {
    local pass="$1" tab
    step "$pass" reports nav nav reports
    step "$pass" reports settle settle
    for tab in performance memory familiarity calibration cognitive runs habits leeches workload; do
        step "$pass" reports "tab_$tab" reports "$tab"
    done
    step "$pass" reports settle2 settle
}

visit_console() {
    local pass="$1"
    step "$pass" console nav nav console
    step "$pass" console settle settle
    step "$pass" console logs logs --tail 50
}

# run_pass PASS: starts the app with the capture, opens the vault, visits the three screens, waits for the
# window to close. Returns 1 when the app did not start answering or did not exit in time.
run_pass() {
    local pass="$1" frames="$out/frames-$pass.csv" pid i=0
    log "pass $pass: starting the app with --frame-times $frames"
    "$bin" --frame-times "$frames" >"$out/app-$pass.log" 2>&1 &
    pid=$!
    until ctl state >/dev/null 2>&1; do
        if ! kill -0 "$pid" 2>/dev/null; then
            log "pass $pass: the app exited before its control socket answered (see app-$pass.log)"
            return 1
        fi
        i=$((i + 1))
        if [ "$i" -ge 120 ]; then
            log "pass $pass: no answer from the control socket after 60 s"
            kill "$pid" 2>/dev/null || true
            return 1
        fi
        sleep 0.5
    done
    log "pass $pass: window answering; opening the vault"
    step "$pass" setup vault_open vault open "$vault"
    visit_rehearsal "$pass"
    log "pass $pass: rehearsal visit done"
    visit_reports "$pass"
    log "pass $pass: reports visit done"
    visit_console "$pass"
    log "pass $pass: console visit done"
    log "pass $pass: close the StudyFrog window now (it writes $frames on exit; up to $close_wait s)"
    i=0
    while kill -0 "$pid" 2>/dev/null; do
        i=$((i + 1))
        if [ "$i" -ge $((close_wait * 2)) ]; then
            log "pass $pass: the window is still open after $close_wait s; stopping it with SIGTERM (no frames file is expected)"
            kill -TERM "$pid" 2>/dev/null || true
            break
        fi
        sleep 0.5
    done
    wait "$pid" 2>/dev/null || true
    if [ -f "$frames" ]; then
        log "pass $pass: frames written: $(($(wc -l <"$frames") - 1)) rows in $frames"
    else
        log "pass $pass: no frames file; the summary will name this pass"
    fi
}

# 1. Machine identity (read-only; needs no display).
log "recording the machine identity into $out/machine.txt"
FRAME_TIME_MACHINE_LABEL="${FRAME_TIME_MACHINE:-unlabelled}"
"$here/bench-machine.sh" "$FRAME_TIME_MACHINE_LABEL" >"$out/machine.txt" 2>"$out/machine-warnings.txt" || true
if [ -s "$out/machine-warnings.txt" ]; then
    while IFS= read -r line; do log "machine: $line"; done <"$out/machine-warnings.txt"
fi

# 2. The passes.
status=0
pass=0
while [ "$pass" -lt "$passes" ]; do
    pass=$((pass + 1))
    run_pass "$pass" || status=1
done

# 3. Statistics: summary.csv and summary.txt, from the frames files (embedded python helper, stdlib only).
python3 -I - "$out" "$passes" "$FRAME_TIME_MACHINE_LABEL" <<'PY' || status=1
import csv
import os
import sys

out, passes, label = sys.argv[1], int(sys.argv[2]), sys.argv[3]
EXPECTED = ["rehearse", "reports", "console"]  # the three screens of TASK-0284, by their ctl names
CAP = 1_000_000  # frame_times::MAX_SAMPLES; a pass that reaches it lost the frames after it


def nearest_rank(n):
    """Rank (from 1) of the 95th percentile: ceil(n * 95 / 100), as frame_times::summary computes it."""
    return -(-n * 95 // 100)


def stats(values):
    """median, p95, worst of microsecond values, with the positions used so a reader can check them."""
    s = sorted(values)
    n = len(s)
    if n % 2:
        median = s[n // 2]
        basis = f"frame {n // 2 + 1} of {n}"
    else:
        lo, hi = n // 2 - 1, n // 2
        median = (s[lo] + s[hi]) // 2
        basis = f"frames {lo + 1} and {hi + 1} of {n}, mean rounded down"
    rank = nearest_rank(n)
    return {"median": median, "basis": basis, "p95": s[rank - 1], "rank": rank, "worst": s[-1]}


def read_identity(path):
    kv = {}
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            for line in f:
                if "=" in line:
                    key, value = line.rstrip("\n").split("=", 1)
                    kv[key] = value
    return kv


identity = read_identity(os.path.join(out, "machine.txt"))
ID_COLS = ["machine", "cpu_model", "commit", "tree_dirty_files", "governor", "boost", "kernel", "rustc"]
id_values = [identity.get(k, "unknown") for k in ID_COLS]

rows, missing, capped = {}, [], []
for p in range(1, passes + 1):
    path = os.path.join(out, f"frames-{p}.csv")
    if not os.path.exists(path):
        missing.append(p)
        continue
    with open(path, newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        if reader.fieldnames != ["frame", "screen", "frame_us"]:
            sys.exit(f"error: {path} has columns {reader.fieldnames}, expected frame,screen,frame_us")
        count = 0
        for r in reader:
            rows.setdefault((p, r["screen"]), []).append(int(r["frame_us"]))
            count += 1
    if count >= CAP:
        capped.append(p)

summary_rows = []
for p in range(1, passes + 1):
    screens = sorted({s for (pp, s) in rows if pp == p} | set(EXPECTED))
    for screen in screens:
        values = rows.get((p, screen), [])
        if values:
            st = stats(values)
            summary_rows.append(id_values + [p, screen, len(values), st["median"], st["p95"], st["rank"], st["worst"]])
        else:
            summary_rows.append(id_values + [p, screen, 0, "", "", "", ""])

with open(os.path.join(out, "summary.csv"), "w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(ID_COLS + ["pass", "screen", "frames", "median_us", "p95_us", "p95_rank", "worst_us"])
    w.writerows(summary_rows)

refused = []
steps_path = os.path.join(out, "steps.csv")
if os.path.exists(steps_path):
    with open(steps_path, newline="", encoding="utf-8") as f:
        for r in csv.DictReader(f):
            if r["ok"] != "1":
                refused.append(f"pass {r['pass']} {r['screen']} {r['step']}: {r['note'].strip()}")

lines = [
    "StudyFrog frame time: CPU time of the UI thread per frame, as the app measures it (microseconds)",
    f"label: {label}",
    "",
    "Machine identity (scripts/bench-machine.sh):",
]
lines += [f"  {k}={v}" for k, v in identity.items()]
lines += [
    "",
    "Statistics rule (the app's frame_times::summary):",
    "  median: sorted frame times; odd n: the middle one; even n: the two middle ones, added and halved (rounded down).",
    "  p95: nearest rank, rank = ceil(n * 95 / 100) counted from 1 in sorted order; p95 is the value at that rank.",
    "  worst: the largest frame time.",
    "  Every frame drawn while the screen showed counts, idle repaints included.",
    "  Each row can be checked from frames-<pass>.csv by filtering on the screen and sorting frame_us.",
    "",
    "Per pass and screen:",
    f"  {'pass':<5}{'screen':<12}{'frames':>8}{'median_us':>12}{'p95_us':>10}{'rank':>8}{'worst_us':>11}",
]
for row in summary_rows:
    p, screen, n = row[len(ID_COLS)], row[len(ID_COLS) + 1], row[len(ID_COLS) + 2]
    if n:
        med, p95, rank, worst = row[len(ID_COLS) + 3], row[len(ID_COLS) + 4], row[len(ID_COLS) + 5], row[len(ID_COLS) + 6]
        lines.append(f"  {p:<5}{screen:<12}{n:>8}{med:>12}{p95:>10}{rank:>8}{worst:>11}")
    else:
        lines.append(f"  {p:<5}{screen:<12}{0:>8}   not measured in this pass")
lines += ["", "Coverage:"]
for p in range(1, passes + 1):
    if p in missing:
        lines.append(f"  pass {p}: no frames file; the window did not close normally. Nothing measured.")
        continue
    gone = [s for s in EXPECTED if not rows.get((p, s))]
    if gone:
        lines.append(f"  pass {p}: not measured: {', '.join(gone)}")
    else:
        lines.append(f"  pass {p}: rehearse, reports and console measured")
    if p in capped:
        lines.append(f"  pass {p}: the capture reached {CAP} frames; later frames were counted, not stored")
lines += ["", "Steps the app refused (steps.csv; the visit went on):"]
lines += [f"  {r}" for r in refused] or ["  none"]
lines += [
    "",
    "Not measured by this run: GPU time, compositor and display latency, input latency.",
    "The frame budget is not set here (STORY-0204): it comes from the owner's baseline.",
]
with open(os.path.join(out, "summary.txt"), "w", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")

print(f"[frame-time] wrote {os.path.join(out, 'summary.csv')}")
print(f"[frame-time] wrote {os.path.join(out, 'summary.txt')}")
if missing:
    print(f"[frame-time] passes without frames: {missing}")
PY

log "done: $out (status $status)"
exit "$status"
