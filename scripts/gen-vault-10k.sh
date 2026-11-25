#!/usr/bin/env bash
# Scratch vault of about 10,000 notes for the frame-time run (EPIC-067, TASK-0284, STORY-0204).
#
# Usage: scripts/gen-vault-10k.sh <out-dir> [notes=10000] [seed=1]
#   Writes <out-dir>/stackNNN/_stack.md (100 stacks) and <out-dir>/stackNNN/noteNNNNN.md. The files are
#   the ones crates/studyfrog-store/benches/vault.rs writes for the store benchmark: the same front matter
#   (id, kind, title, tags, created), the same body with two wiki links, about 1.5 KB per note. The ids come
#   from a generator seeded with <seed>, so one seed always gives the same vault.
#   <out-dir> must not exist or must be empty. About 16 MB; delete it when the run is over.
#   Only notes are written: the rehearsal screen has no cards to rehearse in this vault.
#   Needs bash, python3 (standard library only) and coreutils. Runs anywhere; no display needed.
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 3 ]; then
    echo "usage: $0 <out-dir> [notes=10000] [seed=1]" >&2
    exit 2
fi
out="$1"
notes="${2:-10000}"
seed="${3:-1}"
for n in "$notes" "$seed"; do
    case "$n" in '' | *[!0-9]*) echo "notes and seed must be non-negative integers" >&2; exit 2 ;; esac
done
[ "$notes" -ge 1 ] || { echo "notes must be at least 1" >&2; exit 2; }

if [ -e "$out" ] && [ -n "$(ls -A "$out" 2>/dev/null)" ]; then
    echo "error: $out exists and is not empty; give a new folder" >&2
    exit 1
fi
mkdir -p "$out"

python3 -I - "$out" "$notes" "$seed" <<'PY'
import datetime
import os
import random
import sys

out, count, seed = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
STACKS = 100
ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"  # Crockford base32, as ULID uses
rng = random.Random(seed)
stamp_ms = int(datetime.datetime(2026, 10, 30, 12, tzinfo=datetime.timezone.utc).timestamp() * 1000)


def ulid():
    """A ULID: 48-bit time, then 80 random bits, in 26 characters (the format studyfrog-md writes)."""
    value = (stamp_ms << 80) | rng.getrandbits(80)
    return "".join(ALPHABET[(value >> (5 * (25 - i))) & 31] for i in range(26))


filler = "Retrieval practice beats rereading; spacing the reviews slows forgetting. " * 18

for s in range(STACKS):
    folder = os.path.join(out, f"stack{s:03d}")
    os.makedirs(folder, exist_ok=True)
    with open(os.path.join(folder, "_stack.md"), "w", encoding="utf-8") as f:
        f.write(f"---\nid: {ulid()}\nkind: stack\ntitle: Stack {s}\n---\n")

for n in range(count):
    text = (
        f"---\nid: {ulid()}\nkind: note\ntitle: Note number {n}\n"
        f"tags: [bench, topic{n % 10}]\ncreated: 2026-10-30T12:00:00.000Z\n---\n"
        f"# Note {n}\n\nSee [[Note number {(n + 1) % count}]] and "
        f"[[Note number {(n + 7) % count}#Part|next]].\n\n{filler}\n"
    )
    path = os.path.join(out, f"stack{n % STACKS:03d}", f"note{n:05d}.md")
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

print(f"wrote {count} notes in {STACKS} stacks to {out} (seed {seed})")
PY
