---
id: STORY-0190
type: story
title: "Homework commands in the control protocol"
status: done
epic: EPIC-064
parents: ['EPIC-064']
estimate: 2
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'homework list answers with the items as the app last read them, with urgency, countdown and checklist progress'
  - 'homework new with a title and the flags due, time, estimate, priority, stack and checklist fills in the dialog and saves it; a refusal is an error with the reason'
  - 'homework advance and homework tick work, and homework tab selects a tab; list and tab are reads, the others are writes'
---

## Context
The homework screen, its tabs, the dialog and checklist ticks (STORY-0167) can only be used with the mouse. Agents and scripts need to list, create and move assignments.

## Description
Done in `studyfrog-ui`: `control_words/homework.rs` (words), `control_plan/homework.rs` (the clicks), `control_reply/homework.rs` (the answers and refusals), with the policy classes and `help` rows. Tests: `tests/control_homework.rs`.

## Acceptance Criteria
- [x] homework list answers with the items as the app last read them, with urgency, countdown and checklist progress
- [x] homework new with a title and the flags due, time, estimate, priority, stack and checklist fills in the dialog and saves it; a refusal is an error with the reason
- [x] homework advance and homework tick work, and homework tab selects a tab; list and tab are reads, the others are writes

## Design Notes
The tab and the list answer from the last read; a list taken right after a change may not show it until the app has read it again.

## Verification
Review (2026-11-22): all three criteria MET, checked on the running window (`/tmp/claude-0/review-b`).
- Criterion 1: `ctl "homework list"` answers the items with `urgency` (due_soon), `countdown` ("due in 2 d"), `steps` ({done:0,total:3}).
- Criterion 2: `ctl "homework new \"Essay\" --due 2026-11-25 --time 17:00 --estimate 90 --priority high --stack maths --checklist \"Outline;Draft;Proofread\""` saved the item; the list shows due_date, due_time, estimate_min 90, priority high, course = maths, 3 steps. Refusals: an empty title answers "the title must be one non-empty line" (exit 1); `--time 25:99` answers "the due time must look like 18:30" (exit 1).
- Criterion 3: `homework advance` moved the status not_started -> in_progress -> completed; `homework tick <id> 2` toggled step 2 (done 1 -> 0). `homework tab overdue` and `homework tab all` changed the tab on screen: `/tmp/claude-0/review-b/hw-overdue.png` (Overdue selected, "Nothing here yet") and `/tmp/claude-0/review-b/hw-all.png` (All selected, the two items). `help` classes: list and tab read; new, advance, tick write. Bad tab name is an error listing the tabs.
Note: `homework list` follows the selected tab (under Overdue it answered count 0).

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
