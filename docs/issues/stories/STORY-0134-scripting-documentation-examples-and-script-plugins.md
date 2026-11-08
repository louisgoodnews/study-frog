---
id: STORY-0134
type: story
title: Scripting documentation, examples and script plugins
status: done
epic: EPIC-028
parents: [EPIC-028]
estimate: 3
labels: [m12, docs]
acceptance_criteria:
  - "A help page and a spec describe the Python API, scripts.toml, events, access levels and the console; `examples/scripts/` holds a daily vault health check, an export script, an event-bound script and a script importer, each run by a test"
  - "docs/EXTENDING.md says what the compiled-in registry offers (importers, card types, format adapters) and what scripts add out of process, and what is not possible"
---
## Context
Users need to know how to write and install scripts (EPIC-028).

## Description
Help topic `scripting`, spec `docs/specs/scripting.md`, examples, EXTENDING.md section. Tasks: TASK-0251, TASK-0252.

## Acceptance Criteria
- [x] A help page and a spec describe the Python API, scripts.toml, events, access levels and the console; `examples/scripts/` holds a daily vault health check, an export script, an event-bound script and a script importer, each run by a test
- [x] docs/EXTENDING.md says what the compiled-in registry offers (importers, card types, format adapters) and what scripts add out of process, and what is not possible

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`).

## Progress (2026-11-08)
Built: `docs/help/scripting.md` (registered in `help_pages.rs`, listed in the help README, linked from the FAQ, and
the Console's F1 topic), `docs/specs/scripting.md`, `examples/scripts/` (`health_check.py`, `export_vault.py`,
`tag_new_cards.py`, `import_glossary.py`, `glossary.txt`, `scripts.toml`), the "Script plugins" section and comparison
table in `docs/EXTENDING.md`, `python/README.md`. Verified: `crates/studyfrog-app/tests/script_examples.rs` runs the
health check (read access, and writing its note), the export twice with pruning, the glossary importer twice (3 cards,
then none, a bad line reported), checks the sample `scripts.toml` with `script list` and runs the tagger bound to
`item_created` in a headless instance; the help test checks the page is compiled in and its links resolve. Not
covered: the export example's pruning only sees one file in the test, because exports of the same day share a name.

## Dependencies
STORY-0131, STORY-0132, STORY-0133.

## Open Questions
None.
