#!/usr/bin/env bash
# List .rs files longer than LIMIT lines (default 250). Exit 1 if any are found.
# Usage: scripts/linecount.sh [LIMIT]
set -euo pipefail
cd "$(dirname "$0")/.."
limit="${1:-250}"
over=$(find . -name '*.rs' -not -path './target/*' -not -path './.git/*' -print0 \
  | while IFS= read -r -d '' f; do printf '%s\t%s\n' "$(wc -l < "$f")" "$f"; done \
  | awk -F'\t' -v l="$limit" '$1 > l' | sort -rn)
if [ -n "$over" ]; then
  echo "Files over $limit lines:"
  echo "$over"
  exit 1
fi
echo "linecount: ok (no .rs file over $limit lines)"
