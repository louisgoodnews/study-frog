#!/usr/bin/env bash
# Reference run of the budgeted benchmarks on one machine (EPIC-067, docs/contributing/BENCHMARKING.md).
#
# Usage: scripts/bench-reference.sh <machine-label> <out-dir> [runs=5] [case-slug]
#   Each case runs `runs` times, each time as a separate process (criterion's default sample count).
# The median of each invocation (ns) is appended to <out-dir>/<slug>.ns, one per line; those files are
# what scripts/bench-compare.sh reads. A case whose runs spread more than 5% is extended to 10 runs.
# <out-dir>/machine.txt holds the machine identity and hygiene state (scripts/bench-machine.sh).
# Each invocation also saves a criterion baseline named <label>-run<i>.
#
# Not done here: interleaving two builds (baseline and new, alternating). That is TASK-0279.
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 4 ]; then
    echo "usage: $0 <machine-label> <out-dir> [runs=5] [case-slug]" >&2
    exit 2
fi
label="$1"
out="$2"
runs="${3:-5}"
only="${4:-}"
case "$runs" in
    '' | *[!0-9]*) echo "runs must be a positive integer" >&2; exit 2 ;;
esac
[ "$runs" -ge 1 ] || { echo "runs must be a positive integer" >&2; exit 2; }

cd "$(dirname "$0")/.."
mkdir -p "$out"
scripts/bench-machine.sh "$label" >"$out/machine.txt"

# package|bench|criterion case id (exact)|file slug. The ids are the budgeted cases in BENCHMARKING.md.
CASES='
studyfrog-srs|scheduling|fsrs_schedule_100k|fsrs_schedule_100k
studyfrog-srs|scheduling|sm2_schedule_100k|sm2_schedule_100k
studyfrog-store|vault|vault_10k/scan_parse_only|vault_scan_parse
studyfrog-store|vault|vault_10k/cold_reindex|vault_cold_reindex
studyfrog-store|vault|vault_10k/warm_reindex_no_changes|vault_warm_reindex
studyfrog-graph|layout|graph_5k/layout_step|graph_layout_step
studyfrog-graph|layout|graph_5k/project|graph_project
studyfrog-rehearsal|rehearsal|rehearsal/run_through_1000_items_2001_steps|rehearsal_run_1000
studyfrog-rehearsal|rehearsal|rehearsal/build_queue_10k_spaced|rehearsal_queue_spaced
studyfrog-rehearsal|rehearsal|rehearsal/build_queue_10k_shuffled|rehearsal_queue_shuffled
studyfrog-rehearsal|rehearsal|rehearsal/build_queue_10k_hardest_first|rehearsal_queue_hardest
studyfrog-rehearsal|rehearsal|rehearsal/build_queue_10k_lowest_retrievability|rehearsal_queue_lowest
studyfrog-report|reports|retention curve over the history|report_retention
studyfrog-report|reports|latency stats and histogram over one sample per review|report_latency
studyfrog-report|reports|replay, estimate, forecast and compare (backtest)|report_backtest
studyfrog-report|reports|familiarity of 10,000 items with 25 events each|report_familiarity
studyfrog-report|analytics|survival report (true retention, tier curves, fit)|analytics_survival
studyfrog-report|analytics|chronotype of every review and answer time|analytics_chronotype
studyfrog-report|analytics|fatigue report of one sample per review|analytics_fatigue
studyfrog-report|analytics|leeches of every item|analytics_leeches
studyfrog-report|analytics|confusion pairs of 400 lapsing items|analytics_confusion
studyfrog-report|analytics|forecast of 5,000 cards, 365 days, 16 futures|analytics_forecast_365
studyfrog-report|analytics|cmrr of 5,000 cards, 12 targets, 180 days, 4 futures|analytics_cmrr
studyfrog-pdf|pdf|pdf_500_pages/info|pdf_info
studyfrog-pdf|pdf|pdf_500_pages/all_text|pdf_all_text
studyfrog-pdf|pdf|pdf_500_pages/page_text_with_words|pdf_page_words
studyfrog-pdf|pdf|pdf_500_pages/render_one_page_at_1x|pdf_render_1x
studyfrog-slm|compare|score a 30-word answer, lexical|slm_lexical_30
'

# Median (ns) of one criterion invocation, read from criterion's "time:" line on stdout.
# Lines of the form "change: time: [-1.2% ...]" have no units and are skipped.
run_once() {
    local pkg="$1" bench="$2" id="$3" tag="$4"
    cargo bench -q -p "$pkg" --bench "$bench" -- --exact "$id" --save-baseline "$tag" 2>/dev/null |
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

# Spread (max - min) / mean of the medians in a file; 0 when the file is empty.
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

run_case() {
    local pkg="$1" bench="$2" id="$3" slug="$4" file i m spread
    file="$out/$slug.ns"
    : >"$file"
    cargo bench -q -p "$pkg" --bench "$bench" --no-run >/dev/null 2>&1
    i=0
    while [ "$i" -lt "$runs" ]; do
        i=$((i + 1))
        m="$(run_once "$pkg" "$bench" "$id" "$label-run$i")" || {
            echo "error: no median for '$id' (run $i)" >&2
            exit 1
        }
        echo "$m" >>"$file"
    done
    # The rule in BENCHMARKING.md: a side whose runs spread more than 5% is repeated to 10.
    spread="$(spread_of "$file")"
    if [ "$(awk -v s="$spread" -v n="$i" 'BEGIN { print (n < 10 && s > 0.05) ? "yes" : "no" }')" = yes ]; then
        echo "  $slug: spread $(awk -v s="$spread" 'BEGIN { printf "%.1f", s * 100 }')% over $i runs, repeating to 10" >&2
        while [ "$i" -lt 10 ]; do
            i=$((i + 1))
            m="$(run_once "$pkg" "$bench" "$id" "$label-run$i")" || {
                echo "error: no median for '$id' (run $i)" >&2
                exit 1
            }
            echo "$m" >>"$file"
        done
        spread="$(spread_of "$file")"
    fi
    echo "$slug: runs=$i median_ns=$(sort -g "$file" | awk '{ v[NR] = $1 } END { print (NR % 2) ? v[(NR + 1) / 2] : (v[NR / 2] + v[NR / 2 + 1]) / 2 }') spread=$(awk -v s="$spread" 'BEGIN { printf "%.1f", s * 100 }')%"
}

while IFS='|' read -r pkg bench id slug; do
    [ -n "$pkg" ] || continue
    if [ -n "$only" ] && [ "$only" != "$slug" ]; then
        continue
    fi
    run_case "$pkg" "$bench" "$id" "$slug"
done <<<"$CASES"
