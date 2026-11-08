"""Event-bound: when an item is created, tag it with the day it was made (`added-YYYY-MM`).

Bound to `item_created` in scripts.toml with `access = "write"`. StudyFrog passes the event in
STUDYFROG_EVENT; `studyfrog.trigger_event()` reads it.
"""

from __future__ import annotations

import datetime as dt

import studyfrog


def main() -> None:
    event = studyfrog.trigger_event()
    if event is None or event.kind != "item_created" or not event.data.get("uid"):
        print("not started by an item_created event; nothing to do")
        return
    tag = "added-" + dt.date.today().strftime("%Y-%m")
    with studyfrog.connect() as sf:
        page = sf.open_item(event.data["uid"])["editor"] or {}
        tags = [t for t in (page.get("tags") or "").replace(",", " ").split() if t]
        if tag not in tags:
            sf.edit_item(event.data["uid"], "tags", ", ".join(tags + [tag]))
    print(f"tagged {event.data.get('title')!r} with {tag}")


if __name__ == "__main__":
    main()
