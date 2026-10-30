#!/usr/bin/env bash
# Enforce the crate dependency direction from ADR 0001.
# Every studyfrog-* dependency in a crate manifest must be in the allowed list below.
set -euo pipefail
cd "$(dirname "$0")/.."

declare -A allowed=(
  [core]=""
  [md]="core"
  [store]="core md"
  [srs]="core"
  [rehearsal]="core srs"
  [slm]="core"
  [report]="core srs"
  [achieve]="core"
  [board]="core"
  [calendar]="core"
  [service]="core md store srs rehearsal"
  [ui]="core md store srs rehearsal slm report achieve board calendar service"
  [app]="ui"
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
[ $status -eq 0 ] && echo "check-deps: ok"
exit $status
