#!/usr/bin/env bash
# Enforce the crate dependency direction from ADR 0001.
# Every studyfrog-* dependency in a crate manifest must be in the allowed list below.
set -euo pipefail
cd "$(dirname "$0")/.."

declare -A allowed=(
  [core]=""
  [config]="core"
  [graph]=""
  [md]="core"
  [store]="core md config calendar board"
  [srs]="core"
  [rehearsal]="core srs"
  [slm]="core"
  [candle]="slm"
  [report]="core srs"
  [achieve]="core"
  [board]="core"
  [calendar]="core"
  [ext]=""
  [service]="core md config calendar board store srs rehearsal slm achieve report ext"
  [ui]="core md config graph store srs rehearsal slm report achieve board calendar service ext"
  [app]="ui slm candle ext"
)

status=0
for crate in "${!allowed[@]}"; do
  manifest="crates/studyfrog-$crate/Cargo.toml"
  [ -f "$manifest" ] || { echo "missing $manifest"; status=1; continue; }
  deps=$(sed -n '/^\[\(dev-\|build-\)\?dependencies\]/,/^\[/p' "$manifest" \
    | grep -oE '^studyfrog-[a-z]+' | sed 's/^studyfrog-//' || true)
  for dep in $deps; do
    case " ${allowed[$crate]} " in
      *" $dep "*) ;;
      *) echo "forbidden dependency: studyfrog-$crate -> studyfrog-$dep"; status=1 ;;
    esac
  done
done
# Example extensions may depend on studyfrog-ext and nothing else from this workspace.
for manifest in examples/*/Cargo.toml; do
  [ -f "$manifest" ] || continue
  deps=$(sed -n '/^\[\(dev-\|build-\)\?dependencies\]/,/^\[/p' "$manifest" \
    | grep -oE '^studyfrog-[a-z]+(-[a-z]+)*' | sed 's/^studyfrog-//' || true)
  for dep in $deps; do
    [ "$dep" = "ext" ] || { echo "forbidden dependency: $manifest -> studyfrog-$dep (examples use studyfrog-ext only)"; status=1; }
  done
done
[ $status -eq 0 ] && echo "check-deps: ok"
exit $status
