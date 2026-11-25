#!/usr/bin/env bash
# Profile one screen of the running window (TASK-0286, STORY-0205, EPIC-067). Profile before optimising
# (docs/contributing/BENCHMARKING.md).
#
# Usage: scripts/profile-screen.sh <screen> <out-dir>
#   <screen>   a name `studyfrog ctl nav` accepts: home, library, rehearse, settings, help, calendar,
#              board, reports, agent, run, stack:<folder>, pdf:<path>
#   <out-dir>  created if missing; receives the raw output and summary.txt
#
# Environment (all optional):
#   STUDYFROG_BIN    app binary (default target/release/studyfrog; built with cargo when missing)
#   PROFILE_SECONDS  sampling window in seconds (default 20)
#   PROFILE_FREQ     perf sampling frequency in Hz (default 999)
#   PROFILE_STEPS    file of `studyfrog ctl` requests (one per line, e.g. "nav reports"), sent one per
#                    second while the sampler runs, so the profile covers interaction, not an idle window
#
# Tool choice, first match wins:
#   1. perf          on PATH: `perf record -g -p <pid>` on the running window, driven through
#                    `studyfrog ctl`. Writes perf.data, perf-report.txt (`perf report --stdio
#                    --percent-limit 2`) and summary.txt with the top self-time functions.
#   2. cargo flamegraph  when `cargo flamegraph --help` works: runs the window under it, writes
#                    flamegraph.svg. Needs perf underneath on Linux, so it is rarely reached; no text
#                    summary is produced on this path.
#   3. dhat          when cargo and crates/studyfrog-ui/tests/profile_screen.rs exist: runs that test
#                    with PROFILE_SCREEN=<screen> and PROFILE_OUT_DIR=<out-dir>. The test must install
#                    dhat::Alloc as the global allocator, drive the screen's render loop and write
#                    <out-dir>/dhat-heap.json (see crates/studyfrog-graph/tests/allocs.rs for the
#                    pattern; dhat allows one profiler per process). summary.txt lists the top
#                    allocation sites by bytes from that file, read with python3.
#   Otherwise the script exits 1 and names what to install.
#
# perf record on a process you started needs kernel.perf_event_paranoid <= 2 (or CAP_PERFMON).
# Output lines are prefixed "[profile]" so logs can be grepped.
set -euo pipefail

if [ "$#" -ne 2 ] || [ -z "$1" ] || [ -z "$2" ]; then
    echo "usage: $0 <screen> <out-dir>" >&2
    exit 2
fi
screen="$1"
out="$2"
case "$screen" in
    *[!A-Za-z0-9_:./-]*) echo "[profile] error: screen '$screen' has characters not allowed in a screen name" >&2; exit 2 ;;
esac

cd "$(dirname "$0")/.."
secs="${PROFILE_SECONDS:-20}"
freq="${PROFILE_FREQ:-999}"
bin="${STUDYFROG_BIN:-target/release/studyfrog}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
app_pid=""

log() { echo "[profile] $*"; }

if command -v perf >/dev/null 2>&1; then
    tool=perf
elif command -v cargo >/dev/null 2>&1 && cargo flamegraph --help >/dev/null 2>&1; then
    tool=flamegraph
elif command -v cargo >/dev/null 2>&1 && [ -f crates/studyfrog-ui/tests/profile_screen.rs ]; then
    tool=dhat
else
    log "no profiler available: install perf (or cargo-flamegraph), or add crates/studyfrog-ui/tests/profile_screen.rs and cargo for the dhat fallback"
    exit 1
fi
log "tool: $tool (screen: $screen, out: $out)"

# Waits up to 30 s for the window's control socket to answer.
wait_ctl() {
    local i
    for i in $(seq 1 30); do
        if "$bin" ctl state >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done
    log "error: the window did not answer 'studyfrog ctl state' (see $out/app.log)"
    return 1
}

# Sends each line of PROFILE_STEPS through ctl, one per second.
drive_steps() {
    local line
    [ -n "${PROFILE_STEPS:-}" ] && [ -f "$PROFILE_STEPS" ] || return 0
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        log "ctl $line"
        # shellcheck disable=SC2086
        "$bin" ctl $line >>"$out/ctl.log" 2>&1 || log "warning: ctl '$line' failed"
        sleep 1
    done <"$PROFILE_STEPS"
}

start_app() {
    if [ ! -x "$bin" ]; then
        log "building the app (cargo build --release -p studyfrog-app)"
        cargo build --release -q -p studyfrog-app
    fi
    "$bin" >"$out/app.log" 2>&1 &
    app_pid=$!
    trap 'if [ -n "$app_pid" ]; then kill "$app_pid" 2>/dev/null || true; fi' EXIT
    log "window started (pid $app_pid), waiting for the control socket"
    wait_ctl
    "$bin" ctl nav "$screen" >>"$out/ctl.log" 2>&1 || {
        log "error: could not open screen '$screen' (see $out/ctl.log)"
        exit 1
    }
    sleep 2
}

run_perf() {
    start_app
    log "perf record -g for ${secs}s at ${freq} Hz"
    perf record -g -F "$freq" -p "$app_pid" -o "$out/perf.data" -- sleep "$secs" >"$out/perf-record.log" 2>&1 &
    local rec=$!
    drive_steps
    wait "$rec" || log "warning: perf record exited non-zero (see $out/perf-record.log)"
    log "perf report"
    perf report -i "$out/perf.data" --stdio --percent-limit 2 >"$out/perf-report.txt" 2>&1 || true
    {
        echo "tool: perf"
        echo "screen: $screen"
        echo "sampling: ${secs}s at ${freq} Hz, call graphs on"
        echo "raw: perf.data, perf-report.txt"
        echo "top self-time functions (>= 2%):"
        perf report -i "$out/perf.data" --stdio --no-children -g none --percent-limit 2 2>/dev/null |
            grep -E '^ +[0-9.]+%' | head -15 || true
    } >"$out/summary.txt"
}

run_flamegraph() {
    log "cargo flamegraph: window under perf for ${secs}s"
    # The window is started by cargo flamegraph itself, so the control socket is waited for, not the pid.
    cargo flamegraph --release -p studyfrog-app --bin studyfrog -o "$out/flamegraph.svg" \
        >"$out/flamegraph.log" 2>&1 &
    local fg=$!
    wait_ctl || { kill -INT "$fg" 2>/dev/null || true; exit 1; }
    "$bin" ctl nav "$screen" >>"$out/ctl.log" 2>&1 || true
    drive_steps
    sleep "$secs"
    kill -INT "$fg" 2>/dev/null || true
    wait "$fg" 2>/dev/null || true
    [ -f perf.data ] && mv perf.data "$out/perf.data"
    {
        echo "tool: cargo flamegraph"
        echo "screen: $screen"
        echo "raw: flamegraph.svg, flamegraph.log"
        echo "text summary: not produced on this path; open flamegraph.svg"
    } >"$out/summary.txt"
}

run_dhat() {
    log "dhat: cargo test profile_screen (PROFILE_SCREEN=$screen)"
    PROFILE_SCREEN="$screen" PROFILE_OUT_DIR="$out" \
        cargo test -q -p studyfrog-ui --test profile_screen -- --nocapture >"$out/test.log" 2>&1 || {
        log "error: the dhat test failed (see $out/test.log)"
        exit 1
    }
    [ -f "$out/dhat-heap.json" ] || { log "error: no dhat-heap.json in $out"; exit 1; }
    {
        echo "tool: dhat"
        echo "screen: $screen"
        echo "raw: dhat-heap.json, test.log (open in dh_view.html)"
        echo "top allocation sites by bytes (up to three app frames each):"
        python3 -I - "$out/dhat-heap.json" <<'PY'
import json
import sys

d = json.load(open(sys.argv[1]))
ftbl = d.get("ftbl", [])
pps = d.get("pps", [])
skip = ("alloc::", "std::", "core::", "dhat::", "<alloc", "__rust", "[unknown]")
print(f"total {sum(p.get('tb', 0) for p in pps)} bytes in {sum(p.get('tbk', 0) for p in pps)} blocks")


def name(i):
    f = ftbl[i] if 0 <= i < len(ftbl) else "?"
    return f.split(": ", 1)[1] if ": " in f else f


def site(p):
    names = []
    for i in p.get("fs", []):
        n = name(i)
        if not n.startswith(skip):
            names.append(n)
        if len(names) == 3:
            break
    return " <- ".join(names) or "(no app frame)"


for p in sorted(pps, key=lambda p: p.get("tb", 0), reverse=True)[:15]:
    print(f"{p.get('tb', 0):>12} B {p.get('tbk', 0):>9} blocks  {site(p)[:200]}")
PY
    } >"$out/summary.txt"
}

case "$tool" in
    perf) run_perf ;;
    flamegraph) run_flamegraph ;;
    dhat) run_dhat ;;
esac
log "summary: $out/summary.txt"
cat "$out/summary.txt"
