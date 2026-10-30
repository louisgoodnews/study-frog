#!/usr/bin/env bash
# Regenerate BOARD.md from docs/issues front-matter. Use --check to verify it is current.
# Usage: scripts/board.sh [--check]
set -euo pipefail
cd "$(dirname "$0")/.."
cargo run -q --manifest-path tools/repo-lint/Cargo.toml -- board docs/issues BOARD.md "$@" \
  && echo "board: ok"
