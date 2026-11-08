"""Daily vault health check: counts, overdue and weak items, empty stacks.

Schedule it in scripts.toml (see scripts.toml next to this file). It only reads, so `access = "read"` is
enough. With `--note STACK` (and `access = "write"`) it also writes the report as a note in that stack.
The report goes to standard output, which StudyFrog keeps in its log and shows in the Console.
"""

from __future__ import annotations

import argparse
import datetime as dt

import studyfrog


def report(sf: studyfrog.Client) -> str:
    count = lambda q: len(sf.query(q))  # noqa: E731
    lines = [f"Vault health, {dt.date.today().isoformat()}"]
    lines.append(
        "items: {} cards, {} questions, {} notes".format(
            count("type = card"), count("type = question"), count("type = note")
        )
    )
    lines.append(f"overdue: {count('due < today')}")
    lines.append(f"weak (familiarity under 40 %): {count('familiarity < 0.4')}")
    lines.append(f"never rehearsed: {count('type IN (card, question) AND due IS EMPTY')}")
    empty = [s["title"] for s in sf.stacks() if s["notes"] + s["cards"] + s["questions"] == 0]
    lines.append("empty stacks: " + (", ".join(empty) if empty else "none"))
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--note", metavar="STACK", help="also save the report as a note in this stack")
    args = parser.parse_args()
    with studyfrog.connect() as sf:
        text = report(sf)
        print(text)
        if args.note:
            sf.create_item("note", f"Vault health {dt.date.today().isoformat()}", text=text, stack=args.note)


if __name__ == "__main__":
    main()
