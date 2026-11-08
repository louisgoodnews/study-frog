# Scripts, the Python API and the Console

StudyFrog can be driven by your own scripts: a check every morning, an export every Friday, a tag on every new card,
an importer for a format StudyFrog does not read. Scripts are ordinary programs, usually in Python, that talk to a
running StudyFrog. Nothing has to be compiled and nothing is built into StudyFrog: a script that misbehaves cannot
crash it, and one that runs too long is stopped.

## Where scripts live

Everything is in the `scripts` folder of StudyFrog's home (`studyfrog --print-paths` shows where, usually
`~/.local/share/studyfrog/scripts`):

- your scripts, for example `health_check.py`;
- `scripts.toml`, which says when each script runs;
- `lib/studyfrog/`, the Python API. StudyFrog writes it there itself, so `import studyfrog` works in every script it
  starts. `studyfrog script install-client DIR` copies it somewhere else.

## Writing a script

```python
import studyfrog

with studyfrog.connect() as sf:
    for item in sf.query("type = card AND due < today"):
        print(item.title)
```

`connect()` attaches to the running StudyFrog; a script run by StudyFrog is given its own connection. The API can
read the state and the stacks, run queries (the language of [[query-language]]), create, edit, colour and move items,
create stacks, step through a rehearsal, export and import, read the log and wait for events. Every function waits
until StudyFrog has finished and raises `StudyFrogError` with StudyFrog's reason when something is refused. The full
list is in the README of the Python package and in `studyfrog ctl help`.

## When scripts run: scripts.toml

```toml
[[job]]
name = "health"
script = "health_check.py"
schedule = "30 7 * * *"      # every day at 07:30
access = "read"

[[job]]
name = "tag-new"
script = "tag_new_cards.py"
on = ["item_created"]        # every time an item is created
access = "write"
timeout = 30
```

- **schedule**: five fields, minute hour day month weekday, in your local time. `*` is any, `1-5` a range, `0,30` a
  list, `*/15` every 15, and names work for months and days (`mon-fri`, `nov`). `@hourly`, `@daily`, `@weekly`,
  `@monthly` and `@yearly` are short forms. A job runs once per matching minute; if StudyFrog was not running at the
  time, the job is skipped (up to an hour is caught up after the computer slept).
- **on**: events that start the job: `vault_opened`, `vault_closed`, `item_created`, `item_saved`,
  `rehearsal_finished`, `script_finished`. The script finds the event with `studyfrog.trigger_event()`.
- A job with neither runs only when you start it.
- **args** (a list), **timeout** (seconds, default 60, at most 3600) and **enabled** (`false` switches it off).
- `.py` scripts run with `python3` (or `python = "..."` at the top of the file); other scripts must be executable.

Jobs run while StudyFrog runs: the window, a headless instance, or `studyfrog script daemon` (runs in the background
without a window). `studyfrog script list` shows the jobs and what is wrong with the file. Each run is written to the
log (Settings > Data shows where) with what the script printed; a failed run is logged as an error.

## What a script may do

A script talks to StudyFrog only through its own connection, which allows what its entry says:

- `access = "read"` (the default): read, query and navigate.
- `access = "write"`: also create, edit, move, rehearse, import and export.
- `allow_destructive = true`: also delete. Only add this to scripts you wrote or read yourself.

Scripts are programs you run: on your computer they can do whatever you can do. Only put scripts you trust into the
scripts folder.

## Running a script by hand

- `studyfrog script run health` runs the job named `health`; `studyfrog script run ~/check.py -- arg1 arg2` runs any
  file (with write access unless `--access read` or `--access destructive` says otherwise; `--timeout 30` limits it).
  If StudyFrog is not running, a headless one is started for the run.
- In the **Console** (in the sidebar), `script run health` runs it and shows what it printed when it ends.

## The Console

The Console takes the same commands scripts use, one per line: `state`, `query type = card`, `item create note Ideas
--stack french`, `nav reports`, `logs --tail 20`, `help`. **Enter** sends the line, **Up** and **Down** bring back
earlier lines, **Clear** empties the list. Below the line you see every answer, errors in red, and, as they happen,
the events (an item created, a run finished) and the output of every script that ran.

## Examples

`examples/scripts/` in StudyFrog's source has a daily health check, a weekly export, the tagging of new items, an
importer for glossary files and a `scripts.toml` that uses them.

See also [[import-formats]] and [[faq]].
