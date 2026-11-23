---
id: BUG-0037
type: bug
title: "CSV import into a chosen stack creates a nested stack 'statistik/statistik' instead of using it"
status: done
estimate: 2
labels: [m14, ui, ext]
acceptance_criteria:
  - "Importing a CSV whose stack column names the chosen stack puts its cards in that stack, with no new nested stack"
  - "Importing the same file with 'As the file says' keeps the stack column's own stack"
  - "The Import into menu never lists two stacks that look the same"
---

## Context
Found while auditing STORY-0168 in the running app (2026-11-22). A CSV in the documented format (`stack,kind,title,front,back,tags`, rows with stack `statistik`) was imported with the Import tab's destination set to the stack "Statistik" (`ctl import cards.csv --stack statistik`). The two cards were created in a new nested stack `statistik/statistik` (title "statistik") under Statistik. The Import into menu then listed "Statistik" and "statistik", which look the same. A Markdown deck imported the same way (one card per `## ` heading) went into `statistik/` directly, so the two importers disagree.

Screenshots: `/tmp/claude-0/audit/g5/0168-dest-menu.png` (the duplicate names in the menu); the folders are in `/tmp/claude-0/audit/g5/vault/statistik/statistik/`.

## Description
Decide how the chosen destination and the file's stack column combine (the destination should win, or the column's stack should be used as written, not nested under the destination), and apply the same rule to every importer. Remove the nested stack created by the bad import in the test vault, and check the Import into menu after the fix.

## Acceptance Criteria
- [x] Importing a CSV whose stack column names the chosen stack puts its cards in that stack, with no new nested stack
- [x] Importing the same file with 'As the file says' keeps the stack column's own stack
- [x] The Import into menu never lists two stacks that look the same

## Design Notes
- Touchpoints: `crates/studyfrog-service/src/xfer_plan.rs` (`destination`), `crates/studyfrog-service/src/xfer_csv*.rs`, `crates/studyfrog-ui/src/` (transfer destination menu). See `docs/specs/interchange.md`.
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0168
- BUG-0011 (CSV importer)

## Open Questions
- Which rule the owner wants when the destination and the file's stack differ.

## Review (2026-11-22)

Review (2026-11-22): criterion 1 NOT MET: `ctl import cards.csv --stack statistik` created vault/statistik/statistik again with two cards; the sidebar shows Statistik with a nested statistik (/tmp/claude-0/review-a/import-dialog2.png). Criterion 2 MET: the same file imported with "As the file says" went into vault/statistik, the stack the file names, with no nested stack. Criterion 3 MET for the menu as it now stands: the Import into menu lists Vault root, Biologie, neues-thema and Statistik, with no look-alike pair (/tmp/claude-0/review-a/import-dest.png); nested stacks are not listed there. Status stays backlog (criterion 1 open).

## Fix (2026-11-23)

Cause: `xfer_plan::destination` put every item at `{chosen}/{stack column}`, so a file whose stack column named the chosen stack got a copy nested under it. Fix: `names_chosen` sends an item into the chosen folder when its stack column equals the folder or its last part. Other stacks still nest under the chosen one, and "As the file says" is unchanged.

Test: `studyfrog-service/tests/xfer_destination.rs` (`a_file_naming_the_chosen_stack_lands_in_that_stack_not_a_copy_of_it`, which failed before the fix).

Checked in the running app (Xvfb; `cards.csv` with two `statistik` cards; Import button, destination "Statistik"):
- Before, unmodified build: preview "in statistik/statistik (new)" (`/tmp/claude-0/stderr/before-csv-preview2.png`); import created `vault/statistik/statistik` with both cards.
- After, fixed build: preview "in statistik (new)", "0 new stack(s)" (`/tmp/claude-0/stderr/after-csv-preview.png`); import wrote `vault/statistik/mean.md` and `median.md` with no nested folder (`/tmp/claude-0/stderr/after-csv-result.png`).
- "As the file says" preview: "in statistik (exists at statistik/mean.md)" (`/tmp/claude-0/stderr/after-as-file-says-preview.png`).
- Import into menu: As the file says, Vault root (no stack), Statistik, New stack; no look-alike pair (`/tmp/claude-0/stderr/after-dest-menu.png`).
