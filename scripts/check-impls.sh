#!/usr/bin/env bash
# Enforce P1 (no OOP): flag inherent `impl` blocks. Trait impls (`impl X for Y`) are allowed.
# A genuine exception (e.g. a strictly necessary constructor) needs a line directly above:
#   // allow-impl: <reason>
set -euo pipefail
cd "$(dirname "$0")/.."
status=0
while IFS= read -r file; do
  hits=$(awk -v f="$file" '
    /^[[:space:]]*(pub(\([a-z]+\))? )?(unsafe )?impl[ <]/ && $0 !~ / for / {
      if (prev !~ /allow-impl:/) print f ":" NR ": " $0
    }
    { prev = $0 }' "$file")
  if [ -n "$hits" ]; then echo "$hits"; status=1; fi
done < <(find . -name '*.rs' -not -path './target/*' -not -path './.git/*')
[ $status -eq 0 ] && echo "check-impls: ok (no inherent impl blocks)"
[ $status -eq 0 ] || echo "Inherent impl blocks violate P1; use free functions."
exit $status
