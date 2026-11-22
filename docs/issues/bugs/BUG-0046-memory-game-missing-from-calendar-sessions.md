---
id: BUG-0046
type: bug
title: "The Calendar's session and planned-review rows offer no Memory game entry point"
status: backlog
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

## Description
Add a Memory game start to the Calendar's session rows and to the planned-review rows, beside the existing rehearsal start, using the same stack. Alternatively, change EPIC-059's last criterion to say that the stack page and Rehearse setup are the only launch points.

## Acceptance Criteria
- [ ] A session row on the Calendar offers a Memory game start beside Start rehearsal, or the epic criterion is changed
- [ ] A planned-review row for a stack offers a Memory game start, or the same change is made

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/calendar.rs`, `crates/studyfrog-ui/src/view/calendar_reviews.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-059 (last criterion)
- STORY-0175

## Open Questions
- None.
