---
id: EPIC-032
type: epic
title: Item keys, colorization, and reporting export
status: backlog
parents: []
labels: [m12, core, ui, report]
acceptance_criteria:
  - "Items and stacks support human-readable sequential keys (e.g. STACK_1, FLASHCARD_500) for query filtering, direct navigation, and cross-linking"
  - "Items support an optional color attribute configured via a color picker in the editor and rendered as a tinted background accent in the viewer"
  - "Learning metrics, retention statistics, and rehearsal history from the Reports screen can be exported directly as a CSV file"
---

## Context
As user vaults grow to contain hundreds of cards, notes, and questions across numerous stacks, referencing, organizing, and analyzing study material requires richer visual cues, query ergonomics, and data portability:
1. Long opaque IDs (26-character NanoIDs) are cumbersome to remember or type in query search bars. Users need concise, memorable item keys (e.g. `STACK_1`, `FLASHCARD_500`, `NOTE_12`) for quick search filtering, script referencing, and cross-linking.
2. Learners want visual differentiation across distinct topics, difficulty ratings, or card categories. A color picker allowing item colorization provides instant visual orientation by subtly tinting item backgrounds during rehearsal and browsing.
3. Vault reporting insights (retention rate, mastery breakdown, study time, forecast) are currently trapped inside the UI. Learners need to export reporting data to CSV for personal spreadsheets, statistical tracking, and external graphing.

## Description
Deliver three key vault organization, visual, and reporting enhancements planned for version 0.5.0:
1. Item Keys for Querying (STORY-0096):
   - Generate and maintain sequential, human-readable keys for stacks and items (`STACK_1`, `FLASHCARD_500`, `NOTE_12`, `QUESTION_42`).
   - Enable query search filters by key (`key:FLASHCARD_500` or direct `FLASHCARD_500` jump).
   - Display item keys in the details panel, item header, and browser tables.
2. Item Colorization with Viewer Styling (STORY-0097):
   - Add an optional `color` frontmatter field to items and stacks (supporting hex color codes or curated pastel palette tokens).
   - Provide an intuitive color picker widget in the editor and item details view.
   - Apply the selected color as a tinted background accent in the rehearsal card viewer, editor, and item browser.
3. Reporting Export as CSV (STORY-0098):
   - Add an "Export CSV" action to the Reports screen.
   - Export structured tables including retention rates, mastery distributions, stack review volumes, and time-series rehearsal run history.

## Acceptance Criteria
- [ ] Items and stacks support human-readable sequential keys (e.g. STACK_1, FLASHCARD_500) for query filtering, direct navigation, and cross-linking
- [ ] Items support an optional color attribute configured via a color picker in the editor and rendered as a tinted background accent in the viewer
- [ ] Learning metrics, retention statistics, and rehearsal history from the Reports screen can be exported directly as a CSV file

## Design Notes
Child stories:
- STORY-0096: Item keys for querying (estimate: 5)
- STORY-0097: Item colorization with color picker and viewer background styling (estimate: 3)
- STORY-0098: Reporting export as CSV (estimate: 3)
- STORY-0099: Simplify vault switcher menu (estimate: 3)

Touchpoints across crates:
- `crates/studyfrog-core`: item frontmatter schema (`color`, `key`), query parser extensions in `query.rs`.
- `crates/studyfrog-store`: key allocation and persistence in metadata.
- `crates/studyfrog-service`: `report.rs` (CSV serializer for metrics), `item.rs` (key resolution and color updates).
- `crates/studyfrog-ui`: `view/reports.rs` (export button), `view/editor.rs` (color picker), `view/rehearse/` (background tinting), `view/browser.rs` (key column and search).
- Release target: version 0.5.0 (Phase M12).

## Dependencies
EPIC-015 (Reporting) and EPIC-023 (Usability).

## Open Questions
None.
