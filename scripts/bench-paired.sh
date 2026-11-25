#!/usr/bin/env bash
# Paired comparison of two builds of one benchmark case (EPIC-067, TASK-0279, STORY-0201).
#
# Usage: scripts/bench-paired.sh <package> <bench> <case> <build-A> <build-B> <out-dir> [rounds=5]
#   <package> <bench>  as in `cargo bench -p <package> --bench <bench>`, e.g. studyfrog-srs scheduling
#   <case>             the exact criterion case id, e.g. fsrs_schedule_100k (one case per run)
#   <build-A>          the baseline: a path to a built bench executable, or a git ref (commit, tag, branch)
#   <build-B>          the new build, in the same two forms
#   <out-dir>          results folder, created if missing (layout below)
#   [rounds=5]         each round runs A once, then B once (A, B, A, B, ...). Both sides run the same
#                      number of rounds; the run is extended to 10 when either side spreads over 5%.
#
# A git ref is cloned locally into a scratch directory and built there with `cargo bench --no-run`; the
# executable comes from cargo's JSON output. Builds share CARGO_TARGET_DIR (default <repo>/target), so
# unchanged dependencies are reused. The scratch clone is removed on exit. EPIC-067 prefers that the owner
# builds the baseline from its own checkout and passes the executable path instead.
#
# Each invocation runs the executable directly with --bench --exact <case> and saves a criterion baseline
# named bench-paired-<A|B>-run<i>. Each invocation's median (ns) is read as scripts/bench-reference.sh does.
#
# Results (<out-dir>):
#   machine.txt               machine identity and hygiene state (scripts/bench-machine.sh)
#   builds/A, builds/B        copies of the two executables that were run
#   runs/<A|B>-run<i>.txt     one file per invocation: its median and the criterion lines it was read from
#   <slug>-A.ns, <slug>-B.ns  one median per round (ns), the format bench-reference.sh writes, so
#                             scripts/bench-compare.sh <slug>-A.ns <slug>-B.ns gives the regression verdict
#
# Verdict (printed; exit 1 when B is slower beyond the rule, else 0). The spread rule is the one in
# BENCHMARKING.md and tools/repo-lint bench_compare: floor = max(5%, 3 x the run-to-run spread of A), where
# spread = (max - min) / mean of a side's medians. The difference of the two medians is flagged only when its
# size exceeds the floor. The median of the per-round differences is shown beside it.
#
# Environment: BENCH_LABEL (machine label for machine.txt, default "paired").
set -euo pipefail

usage() { echo "usage: $0 <package> <bench> <case> <build-A> <build-B> <out-dir> [rounds=5]" >&2; exit 2; }
[ "$#" -ge 6 ] && [ "$#" -le 7 ] || usage
pkg="$1"; bench="$2"; case_id="$3"; spec_a="$4"; spec_b="$5"; out="$6"; rounds="${7:-5}"
case "$rounds" in '' | *[!0-9]*) echo "rounds must be a positive integer" >&2; exit 2 ;; esac
[ "$rounds" -ge 1 ] || { echo "rounds must be a positive integer" >&2; exit 2; }
label="${BENCH_LABEL:-paired}"

# Paths given relative to the caller's directory must stay valid after the cd below.
absolute() { case "$1" in /*) printf '%s' "$1" ;; *) printf '%s/%s' "$PWD" "$1" ;; esac; }
if [ -f "$spec_a" ]; then spec_a="$(absolute "$spec_a")"; fi
if [ -f "$spec_b" ]; then spec_b="$(absolute "$spec_b")"; fi
out="$(absolute "$out")"

cd "$(dirname "$0")/.."
repo="$(pwd)"
slug="$(printf '%s' "$case_id" | tr -c 'A-Za-z0-9_' '_')"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$out/runs" "$out/builds"
scripts/bench-machine.sh "$label" >"$out/machine.txt"

# Executable of the bench target, from cargo's JSON messages (stdout only; progress goes to stderr).
build_ref() {
    local ref="$1" dir exe
    dir="$scratch/src-$RANDOM"
    git clone -q --local --no-checkout "$repo" "$dir"
    git -C "$dir" checkout -q --detach "$ref" || { echo "error: no git ref '$ref'" >&2; return 1; }
    (cd "$dir" && CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo/target}" \
        cargo bench -p "$pkg" --bench "$bench" --no-run --message-format=json) >"$scratch/build.json"
    exe="$(python3 -I -c '
import json, sys
for line in open(sys.argv[1]):
    try:
        m = json.loads(line)
    except ValueError:
        continue
    t = m.get("target", {})
    if m.get("reason") == "compiler-artifact" and m.get("executable") and t.get("name") == sys.argv[2] and "bench" in t.get("kind", []):
        print(m["executable"])
        break
' "$scratch/build.json" "$bench")"
    [ -n "$exe" ] || { echo "error: cargo reported no bench executable for '$ref'" >&2; return 1; }
    rm -rf "$dir"
    printf '%s' "$exe"
}

# Copy the executable into the results folder; print the copy's path.
resolve_build() {
    local spec="$1" side="$2" exe
    if [ -f "$spec" ] && [ -x "$spec" ]; then exe="$spec"; else exe="$(build_ref "$spec")"; fi
    cp "$exe" "$out/builds/$side"
    printf '%s' "$out/builds/$side"
}

bin_a="$(resolve_build "$spec_a" A)"
bin_b="$(resolve_build "$spec_b" B)"

# Median (ns) read from criterion's "time:" line on stdout: the middle value of "[low unit median unit high
# unit]". "change:" lines have no units and are skipped.
parse_median() {
    awk '
        index($0, "time:") && index($0, "[") {
            s = $0; sub(/.*\[/, "", s); sub(/\].*/, "", s)
            n = split(s, f, " ")
            if (n == 6 && f[3] ~ /^[0-9.]+$/) {
                u = f[4]; m = 0
                if (u == "ns") m = 1
                else if (u == "ms") m = 1e6
                else if (u == "s") m = 1e9
                else if (u == "us" || u == "µs") m = 1e3
                if (m > 0) { printf "%.6f\n", f[3] * m; found = 1; exit }
            }
        }
        END { if (!found) exit 1 }'
}

# One invocation of a side, run as its own process. Prints its median (ns).
run_once() {
    local side="$1" bin="$2" i="$3" raw median
    raw="$("$bin" --bench --exact "$case_id" --save-baseline "bench-paired-$side-run$i" 2>/dev/null)" || return 1
    median="$(printf '%s\n' "$raw" | parse_median)" || return 1
    {
        echo "side=$side run=$i median_ns=$median binary=$bin"
        printf '%s\n' "$raw"
    } >"$out/runs/$side-run$i.txt"
    printf '%s\n' "$median"
}

# Spread (max - min) / mean of the values in a file (the definition bench-reference.sh uses).
spread_of() {
    awk '
        { v[NR] = $1; sum += $1 }
        END {
            if (NR == 0) { print 0; exit }
            mn = v[1]; mx = v[1]
            for (i = 1; i <= NR; i++) { if (v[i] < mn) mn = v[i]; if (v[i] > mx) mx = v[i] }
            printf "%.6f\n", (mx - mn) / (sum / NR)
        }' "$1"
}

# Median of the values in a file.
median_of() {
    sort -g "$1" | awk '{ v[NR] = $1 } END { print (NR % 2) ? v[(NR + 1) / 2] : (v[NR / 2] + v[NR / 2 + 1]) / 2 }'
}

file_a="$out/$slug-A.ns"
file_b="$out/$slug-B.ns"
: >"$file_a"
: >"$file_b"

# One round: A, then B. A failed invocation stops the script.
round() {
    local i="$1" m
    m="$(run_once A "$bin_a" "$i")" || { echo "error: A run $i failed for '$case_id'" >&2; exit 1; }
    echo "$m" >>"$file_a"
    m="$(run_once B "$bin_b" "$i")" || { echo "error: B run $i failed for '$case_id'" >&2; exit 1; }
    echo "$m" >>"$file_b"
}

i=0
while [ "$i" -lt "$rounds" ]; do
    i=$((i + 1))
    round "$i"
done

# The rule in BENCHMARKING.md: a side whose runs spread over 5% is repeated to 10 rounds.
over_5() { [ "$(awk -v s="$(spread_of "$1")" 'BEGIN { print (s > 0.05) ? 1 : 0 }')" = 1 ]; }
if [ "$i" -lt 10 ] && { over_5 "$file_a" || over_5 "$file_b"; }; then
    echo "spread above 5% (A $(spread_of "$file_a" | awk '{ printf "%.1f", $1 * 100 }')%, B $(spread_of "$file_b" | awk '{ printf "%.1f", $1 * 100 }')%): extending to 10 rounds" >&2
    while [ "$i" -lt 10 ]; do
        i=$((i + 1))
        round "$i"
    done
fi

med_a="$(median_of "$file_a")"
med_b="$(median_of "$file_b")"
spread_a="$(spread_of "$file_a")"
spread_b="$(spread_of "$file_b")"
# Per-round differences (B relative to A), in percent, and their median.
round_diffs="$(paste -d' ' "$file_a" "$file_b" | awk '{ printf "%.4f\n", ($2 - $1) / $1 * 100 }')"
round_median="$(printf '%s\n' "$round_diffs" | sort -g | awk '{ v[NR] = $1 } END { print (NR % 2) ? v[(NR + 1) / 2] : (v[NR / 2] + v[NR / 2 + 1]) / 2 }')"

echo "case=$case_id rounds=$i label=$label out=$out"
# awk exits 1 only for a slowdown beyond the rule (the regression case).
flag=0
awk -v a="$med_a" -v b="$med_b" -v sa="$spread_a" -v sb="$spread_b" -v rm="$round_median" -v n="$i" '
    BEGIN {
        diff = (b - a) / a
        floor = 0.05; if (3 * sa > floor) floor = 3 * sa
        flagged = (diff > floor || -diff > floor) ? 1 : 0
        if (flagged && diff > 0) verdict = "B is SLOWER than A beyond the spread rule: REGRESSION FLAG"
        else if (flagged) verdict = "B is faster than A beyond the spread rule (not a regression)"
        else verdict = "within the spread rule: not flagged"
        printf "A median_ns=%.0f spread=%.1f%%\n", a, sa * 100
        printf "B median_ns=%.0f spread=%.1f%%\n", b, sb * 100
        printf "paired difference B vs A: %+.2f%% (median of %d per-round differences: %+.2f%%)\n", diff * 100, n, rm
        printf "spread rule: floor = max(5%%, 3 x spread of A) = %.1f%%\n", floor * 100
        printf "verdict: %s\n", verdict
        exit (flagged && diff > 0) ? 1 : 0
    }' || flag=$?

echo "feed bench-compare: scripts/bench-compare.sh $file_a $file_b"
exit "$flag"
