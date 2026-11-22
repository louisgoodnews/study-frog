---
id: BUG-0046
type: bug
title: "The Calendar's session and planned-review rows offer no Memory game entry point"
status: done
estimate: 2
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "A session row on the Calendar (Sessions on <date>) offers a Memory game start alongside Start rehearsal, or the epic's entry-point criterion is changed to say sessions are not a launch point"
  - "A planned-review row for a stack (Rehearse <stack>) offers a Memory game start, or the same change is made to the epic"
---

## Context
Found while verifying EPIC-059 (2026-11-22, `/tmp/claude-0/verifyB`). Its last criterion asks for a rehearsal entry point "from stacks and sessions". The stack page has "Memory game" and the Rehearse setup has "Memory game". The Calendar's "Sessions on <date>" panel offers only:
- for a session row: "Start rehearsal for <title>", "Edit", "Skip today" (`crates/studyfrog-ui/src/view/calendar.rs`, `occurrence_row`);
- for a planned-review row: "Rehearse <stack>", "Open <stack>", "Move reviews" (`/tmp/claude-0/verifyB/c02-session.png`).

No Memory game start is offered on either.

**Fix (2026-11-22).** Each session row (for a Study session with a stack) and each planned-review row for a stack offers a "Memory game for <title>" button beside its other actions. It opens the rehearse screen and starts a memory game with the cards of that stack, using the same route as the stack page's button (`view/memory/entry.rs`, `planner_rehearse::memory`, `PlanMsg::Memory`). Test: `studyfrog-ui/tests/memory_entry.rs` `the_memory_game_is_offered_on_a_session_row` and `the_memory_game_is_offered_on_a_planned_review_row`. Running app: before, the session row had Start rehearsal, Edit and Delete only (`/tmp/claude-0/memory/before-cal-session.png`); after, "Memory game for Study" sits beside Start rehearsal (`/tmp/claude-0/memory/after-cal-session.png`), and it opens the board for stack X (`/tmp/claude-0/memory/after-calendar-memory-board.png`). The planned-review row for X offers "Memory game for X" (`/tmp/claude-0/memory/after-cal-review-row.png`) and opens the board (`/tmp/claude-0/memory/after-review-row-memory-board.png`).

## Description
Add a Memory game start to the Calendar's session rows and to the planned-review rows, beside the existing rehearsal start, using the same stack. Alternatively, change EPIC-059's last criterion to say that the stack page and Rehearse setup are the only launch points.

## Acceptance Criteria
- [x] A session row on the Calendar offers a Memory game start beside Start rehearsal, or the epic criterion is changed
- [x] A planned-review row for a stack offers a Memory game start, or the same change is made

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/calendar.rs`, `crates/studyfrog-ui/src/view/calendar_reviews.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-059 (last criterion)
- STORY-0175

## Verification (2026-11-22)

Review (2026-11-22): NOT FIXED. Criterion 1: the session row offers no Memory game start (`/tmp/claude-0/review-c/shots/22-session-row.png`). Criterion 2: the planned-review rows offer no Memory game start (`/tmp/claude-0/review-c/shots/19-cal24.png`). Stays backlog.

## Open Questions
- None.
