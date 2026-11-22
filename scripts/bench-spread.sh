#!/usr/bin/env bash
# Run-to-run spread of one criterion benchmark on an idle machine (EPIC-067).
#
# Usage: scripts/bench-spread.sh <package> <bench> <case> [runs]
#   e.g. scripts/bench-spread.sh studyfrog-srs scheduling fsrs_schedule_100k 10
#
# Each run is criterion's default sample count. For every run the median time of <case> is read
# (converted to milliseconds), then the mean, the coefficient of variation and the spread
# ((max - min) / mean) are printed. The spread sets the noise floor in the regression rule.
set -euo pipefail

if [ "$#" -lt 3 ]; then
    echo "usage: $0 <package> <bench> <case> [runs]" >&2
    exit 2
fi
pkg="$1"; bench="$2"; case_name="$3"; runs="${4:-10}"

cargo bench -p "$pkg" --bench "$bench" --no-run >/dev/null 2>&1

samples="$(mktemp)"
trap 'rm -f "$samples"' EXIT
for _ in $(seq 1 "$runs"); do
    # criterion: "<case>  time:   [low unit median unit high unit]"; the median is the 5th field.
    cargo bench -p "$pkg" --bench "$bench" -- "^${case_name}\$" 2>/dev/null \
        | grep -E "^${case_name}[[:space:]]+time:" | head -1 \
        | awk '{ v=$5; u=$6; if (u=="ns") v/=1e6; else if (u=="us" || u=="µs") v/=1e3; else if (u=="s") v*=1e3; print v }' \
        >>"$samples"
done

if [ ! -s "$samples" ]; then
    echo "no samples for '$case_name' in $pkg/$bench" >&2
    exit 1
fi

awk -v case_name="$case_name" '
    { v[NR] = $1 }
    END {
        n = NR; s = 0
        for (i = 1; i <= n; i++) s += v[i]
        m = s / n
        ss = 0
        for (i = 1; i <= n; i++) ss += (v[i] - m) ^ 2
        sd = (n > 1) ? sqrt(ss / (n - 1)) : 0
        mn = v[1]; mx = v[1]
        for (i = 1; i <= n; i++) { if (v[i] < mn) mn = v[i]; if (v[i] > mx) mx = v[i] }
        printf "%s: runs=%d mean=%.4f ms cv=%.2f%% spread=%.2f%%\n", case_name, n, m, 100 * sd / m, 100 * (mx - mn) / m
    }' "$samples"
