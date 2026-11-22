#!/usr/bin/env bash
# Regression verdict for two benchmark runs (EPIC-067, docs/contributing/BENCHMARKING.md).
#
# Usage: scripts/bench-compare.sh <baseline.ns> <new.ns>
#   Each file holds one run's median per line, in nanoseconds (scripts/bench-reference.sh writes them).
# Prints one verdict line. Exit 0 for no change or an improvement, 1 for a regression, 2 for bad input.
# The rule is implemented in tools/repo-lint/src/bench_compare.rs and described in BENCHMARKING.md.
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run -q --manifest-path tools/repo-lint/Cargo.toml -- bench-compare "$@"
