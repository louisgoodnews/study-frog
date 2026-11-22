---
id: BUG-0037
type: bug
title: "CSV import into a chosen stack creates a nested stack 'statistik/statistik' instead of using it"
status: backlog
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
- [ ] Importing a CSV whose stack column names the chosen stack puts its cards in that stack, with no new nested stack
- [ ] Importing the same file with 'As the file says' keeps the stack column's own stack
- [ ] The Import into menu never lists two stacks that look the same

## Design Notes
- Touchpoints: `crates/studyfrog-service/src/xfer_plan.rs` (`destination`), `crates/studyfrog-service/src/xfer_csv*.rs`, `crates/studyfrog-ui/src/` (transfer destination menu). See `docs/specs/interchange.md`.
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0168
- BUG-0011 (CSV importer)

## Open Questions
- Which rule the owner wants when the destination and the file's stack differ.
