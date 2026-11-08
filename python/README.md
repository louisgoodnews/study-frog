# studyfrog: the Python API

A small client of StudyFrog's control protocol (ADR 0013). Standard library only, Python 3.9 or newer,
Linux or macOS for the socket (the started-instance mode works wherever `studyfrog` runs).

```python
import studyfrog

with studyfrog.connect() as sf:                     # a running StudyFrog, else a headless one
    uid = sf.create_item("card", "der Hund", text="dog", answer="der Hund", stack="german")
    for item in sf.query("type = card and status = new order by title"):
        print(item.key, item.title)
```

## Getting it

- Scripts run by StudyFrog (`studyfrog script run`, scheduled and event-bound jobs, the Console) find it
  without installing anything: the binary writes this package to `<app home>/scripts/lib/` and puts that
  folder on `PYTHONPATH`. `studyfrog script install-client [DIR]` writes it somewhere else.
- From a checkout: `PYTHONPATH=python python3 your_script.py`.

## Connecting

`connect(socket=None, *, home=None, start=True, vault=None, binary=None, read_only=False, allow_destructive=False)`

1. `socket`, else `$STUDYFROG_SOCKET` (set for scripts StudyFrog starts: a socket limited to the script's
   access), else `<app home>/run/control.sock` of a running window or headless instance.
2. If nothing answers and `start` is true, it starts `studyfrog --control-stdio --no-control-socket`
   (`binary`, else `$STUDYFROG_BIN`, else `studyfrog` on the `PATH`) on `vault` (else the vault opened
   last) and talks over its standard streams; closing the client stops it.

`home` names the application home (default `$STUDYFROG_HOME`, else `~/.local/share/studyfrog`).

## The API

Every call waits until the instance has settled and returns its answer; an error answer raises
`StudyFrogError` with StudyFrog's reason.

| Area | Functions |
|---|---|
| reading | `state()`, `stacks()`, `query(expr) -> list[Item]`, `search(text)`, `notices()`, `help()` |
| navigation | `nav(screen)`, `back()`, `forward()`, `settle()` |
| vault | `open_vault(folder)`, `close_vault()` |
| items | `create_item(kind, title, text=, answer=, tags=, stack=) -> uid`, `open_item(ref)`, `edit_item(ref, field, text)`, `color_item(ref, colour)`, `move_item(ref, stack)`, `delete_item(ref)` (destructive) |
| stacks | `create_stack(title, parent=)`, `open_stack(folder)`, `stack_settings(folder)`, `move_stack(folder, into=)` |
| rehearsal | `rehearse_setup(preset=, mode=, limit=, stacks=)`, `rehearse(step, *args)`, `run()` |
| transfer | `export(fmt, dest, stack=, scheduling=)`, `import_file(path, stack=)` |
| log | `logs(tail=50, level=None) -> list[LogEntry]` |
| events | `subscribe(*kinds)`, then `events(timeout=None) -> Iterator[Event]` |
| anything else | `call(cmd, *args)`: any control command (`studyfrog ctl help` lists them) |

Items are named by ULID or key (`FLASHCARD_12`). Queries use the Library's language (`docs/specs/query.md`).

Events: `vault_opened`, `vault_closed`, `item_created`, `item_saved`, `rehearsal_finished`,
`script_finished`. Use a connection of its own for `subscribe` when you also send commands from another
thread.

In a script started by StudyFrog, `trigger_event()` is the event that started it (or `None`),
`job_name()` its `scripts.toml` entry and `access()` what its socket allows (`read`, `write`,
`destructive`).

## Tests

`python/tests/` is a `unittest` suite against the real binary; `cargo test -p studyfrog-app --test
python_client` runs it (`STUDYFROG_BIN` names the binary, every test uses a temporary home).
