#!/usr/bin/env bash
# Validate docs/issues front-matter, naming, hierarchy and section order.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo run -q --manifest-path tools/repo-lint/Cargo.toml -- issues docs/issues \
  && echo "check-issues: ok"
