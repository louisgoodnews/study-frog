#!/usr/bin/env bash
# Enforce P1 (no OOP): fail on inherent `impl` blocks, including macro-generated ones.
# Trait impls (`impl X for Y`) are allowed. A reviewed exception needs the line directly above:
#   // allow-impl: <reason>
# Implemented with syn in tools/repo-lint.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo run -q --manifest-path tools/repo-lint/Cargo.toml -- impls . \
  && echo "check-impls: ok (no inherent impl blocks)" \
  || { echo "Inherent impl blocks violate P1; use free functions."; exit 1; }
