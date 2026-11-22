#!/usr/bin/env bash
# Machine identity and hygiene report for a benchmark run (EPIC-067, docs/contributing/BENCHMARKING.md).
#
# Usage: scripts/bench-machine.sh <machine-label>
#   Prints key=value lines on stdout; warnings for hygiene items not met go to stderr with a
# "warning:" prefix. Read-only: it changes nothing (the governor, boost and SMT settings need root and
# are set by the owner before a run). The hostname and user name are deliberately not recorded.
set -euo pipefail

if [ "$#" -ne 1 ] || [ -z "$1" ]; then
    echo "usage: $0 <machine-label>" >&2
    exit 2
fi
label="$1"
cd "$(dirname "$0")/.."

read_or() { cat "$1" 2>/dev/null || echo "$2"; }

cpu_model="$(awk -F': ' '/^model name/ { print $2; exit }' /proc/cpuinfo 2>/dev/null || true)"
governor="$(read_or /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor unavailable)"
if [ -r /sys/devices/system/cpu/intel_pstate/no_turbo ]; then
    boost="$([ "$(cat /sys/devices/system/cpu/intel_pstate/no_turbo)" = 1 ] && echo disabled || echo enabled)"
else
    boost="$(read_or /sys/devices/system/cpu/cpufreq/boost unavailable)"
    [ "$boost" = 1 ] && boost=enabled || { [ "$boost" = 0 ] && boost=disabled || true; }
fi
smt="$(read_or /sys/devices/system/cpu/smt/control unavailable)"
load_1m="$(awk '{ print $1 }' /proc/loadavg 2>/dev/null || echo unknown)"
busy="$(ps -eo pcpu --no-headers 2>/dev/null | awk '$1 > 10 { n++ } END { print n + 0 }')"
dirty="$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"

cat <<EOF
machine=$label
cpu_model=${cpu_model:-unknown}
logical_cpus=$(nproc)
kernel=$(uname -r)
rustc=$(rustc -V 2>/dev/null || echo unknown)
commit=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
tree_dirty_files=$dirty
governor=$governor
boost=$boost
smt=$smt
load_1m=$load_1m
busy_processes_over_10pct=$busy
utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)
EOF

warn() { echo "warning: $*" >&2; }
[ "$governor" = performance ] || warn "CPU governor is '$governor', not 'performance'"
[ "$boost" = disabled ] || warn "turbo boost is '$boost', not disabled"
[ "$busy" -eq 0 ] || warn "$busy processes use more than 10% CPU"
[ "$dirty" -eq 0 ] || warn "$dirty uncommitted files; the commit alone does not describe the build"
exit 0
