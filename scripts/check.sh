#!/usr/bin/env bash
# Full local gate: format, lints, tests, file size, layering, no-OOP scan, issues, board.
set -euo pipefail
cd "$(dirname "$0")/.."

step() { printf '\n==> %s\n' "$*"; }

step "cargo fmt --check";            cargo fmt --all --check
TOOL=tools/repo-lint/Cargo.toml
step "repo-lint fmt + clippy";       cargo fmt --manifest-path $TOOL --check
cargo clippy -q --manifest-path $TOOL -- -D warnings
step "cargo clippy -D warnings";     cargo clippy --workspace --all-targets -- -D warnings
step "cargo test";                   cargo test --workspace
step "line count (max 250)";         scripts/linecount.sh 250
step "dependency direction";         scripts/check-deps.sh
step "no inherent impl blocks";      scripts/check-impls.sh
step "issue front-matter";           scripts/check-issues.sh
step "BOARD.md is current + WIP";    scripts/board.sh --check

printf '\ncheck: all gates passed\n'
