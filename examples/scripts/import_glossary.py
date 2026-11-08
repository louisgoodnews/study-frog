"""A script importer: turn a glossary file (`term — definition` or `term: definition` per line) into
cards in a stack. This is how a new import format is added without recompiling StudyFrog.

    studyfrog script run examples/scripts/import_glossary.py -- glossary.txt --stack french

Lines starting with `#` and blank lines are skipped; a line without a separator is reported and skipped.
Needs `access = "write"`.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from typing import Iterator, Tuple

import studyfrog

SEPARATOR = re.compile(r"\s+[—–-]\s+|:\s+|\t")


def entries(text: str) -> Iterator[Tuple[int, str, str]]:
    """(line number, term, definition) for each usable line; problems go to standard error."""
    for n, line in enumerate(text.splitlines(), start=1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = SEPARATOR.split(line, maxsplit=1)
        if len(parts) != 2 or not parts[0].strip() or not parts[1].strip():
            print(f"line {n}: no term and definition, skipped", file=__import__("sys").stderr)
            continue
        yield n, parts[0].strip(), parts[1].strip()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("file", type=Path)
    parser.add_argument("--stack", required=True, help="the stack folder to add the cards to")
    parser.add_argument("--tag", default="glossary")
    args = parser.parse_args()
    found = list(entries(args.file.read_text(encoding="utf-8")))
    with studyfrog.connect() as sf:
        known = {i.title for i in sf.query(f"stack = {args.stack} AND type = card")}
        made = 0
        for _, term, definition in found:
            if term in known:
                continue
            sf.create_item("card", term, text=term, answer=definition, tags=[args.tag], stack=args.stack)
            made += 1
    print(f"{made} card(s) created, {len(found) - made} already there")


if __name__ == "__main__":
    main()
