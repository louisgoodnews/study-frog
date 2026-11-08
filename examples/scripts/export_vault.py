"""Export the vault (or one stack) to a folder, keeping the last N exports.

    studyfrog script run examples/scripts/export_vault.py -- ~/Backups/studyfrog --format json --keep 7

Needs `access = "write"` (an export writes files). Scheduled weekly in scripts.toml.
"""

from __future__ import annotations

import argparse
from pathlib import Path

import studyfrog


def prune(folder: Path, keep: int) -> list[Path]:
    """Remove all but the newest `keep` exports in `folder`; the removed files."""
    files = sorted((p for p in folder.iterdir() if p.name.startswith("studyfrog")), key=lambda p: p.stat().st_mtime)
    old = files[: max(0, len(files) - keep)]
    for p in old:
        p.unlink()
    return old


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dest", type=Path)
    parser.add_argument("--format", default="json", choices=["json", "toml", "ron", "csv", "markdown"])
    parser.add_argument("--stack")
    parser.add_argument("--keep", type=int, default=0, help="keep only the newest N exports (0: all)")
    args = parser.parse_args()
    args.dest.mkdir(parents=True, exist_ok=True)
    before = {p: p.stat().st_mtime_ns for p in args.dest.iterdir()}
    with studyfrog.connect() as sf:
        sf.export(args.format, args.dest, stack=args.stack, scheduling=True)
    new = sorted(p for p in args.dest.iterdir() if before.get(p) != p.stat().st_mtime_ns)
    print("exported:", ", ".join(p.name for p in new) or "nothing")
    if args.keep:
        removed = prune(args.dest, args.keep)
        print(f"removed {len(removed)} older export(s)")


if __name__ == "__main__":
    main()
