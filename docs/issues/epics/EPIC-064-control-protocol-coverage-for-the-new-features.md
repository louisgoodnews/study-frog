---
id: EPIC-064
type: epic
title: "CLI control protocol coverage for the new features"
status: review
parents: []
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'Every feature shipped in the v1.0 milestones can be driven through the control protocol, with a help row and a test per command family'
  - 'Each new feature family has its own story under this epic, and the epic is continuous: stories are added as features land'
  - 'Read commands answer with the data they read; write commands are refused with the reason when the app refuses them'
---

## Context
EPIC-038 built the control protocol for the features of its time. The features added since (homework, the import destinations and PDF cards, search focus, tooltips, appearance and the calendar's homework markers) cannot yet be driven from the command line, so an agent or a script cannot test or use them. This epic is continuous: each new feature family gets a story here as it lands.

## Description
- Add command families to `control_words`, `control_plan`, `control_reply`, `control_policy` and `control_help`, in submodules of those files (the `lib.rs` module list is at its size limit).
- Keep the protocol's rules: reads answer with data, writes are classed in the policy and refused with the reason when the app refuses them, and every command has a `help` row.
- Test each family the way `tests/control_homework.rs` does: words, class, plan and answer.

## Acceptance Criteria
- [ ] Every feature shipped in the v1.0 milestones can be driven through the control protocol, with a help row and a test per command family
- [x] Each new feature family has its own story under this epic, and the epic is continuous: stories are added as features land
- [x] Read commands answer with the data they read; write commands are refused with the reason when the app refuses them

## Design Notes
A command family is done when its commands parse, plan, answer and appear in `help`, and its tests pass. Coverage is measured against the stories of the other epics, not against the code.

## Verification
Review (2026-11-22): criterion 1 NOT MET: homework, appearance, calendar and search/tooltips are driven and have help rows, but the features of the v1.0 milestones include board, diary, reports, achievements, graph, assistant and match, which are reachable only by `nav` (STORY-0124's open list). Criterion 2 MET: stories STORY-0190 to STORY-0194 (and STORY-0191, import destinations and PDF cards) exist under this epic; continuity is ongoing. Criterion 3 NOT MET: write refusals answer with the reason (homework new, tick, create question, theme, density) and reads answer with data (homework list, calendar homework, reminders due), but `homework tab` and `calendar view`/`period` answer with the whole state, which has no homework tab or planner view field, so the read's data is not in the answer (`ctl state` has no `homework` or `planner` key).

Review (2026-11-23): criterion 3 MET: `homework tab overdue` answers `{"tab": "overdue", "subject": null}` (the tab in front, not the whole state); `calendar period 2026-11-25` answers `{"view": "week", "month": "2026-11", "selected": "2026-11-25", "sessions": []}` and a bare `calendar view` repeats it; `board column Backlog` answers its six cards with status Backlog; `diary list 2026-11-23` answers the entry with its fields; `reports memory` answers retention buckets, latency and forecast. A refused write answers with the reason: `board add "Lost card" --column Nowhere` (test `control_board.rs`), and `board column Nowhere` and `diary open <unknown ULID>` answer with errors. Criterion 1 NOT MET: board, diary, reports and the calendar views are now driven with help rows and a test per family (`control_board.rs`, `control_diary.rs`, `control_reports.rs`, `control_calendar_view.rs`); achievements, graph, assistant, settings, match, folders and PDF are still reachable only by `nav`. Criterion 2 MET (unchanged).

## Dependencies
Extends EPIC-038 (the protocol itself).

## Open Questions
None.
