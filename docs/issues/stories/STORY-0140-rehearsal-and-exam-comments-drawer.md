---
id: STORY-0140
type: story
title: Expandable comments drawer in rehearsal and exam modes
status: done
epic: EPIC-043
parents: [EPIC-043]
estimate: 3
labels: [m12, ui, rehearsal]
acceptance_criteria:
  - "Rehearsal and exam mode views provide an optional expandable drawer or slide-out panel allowing learners to view and add comments for the current card on screen"
  - "The comments drawer displays chronological comments, author tags (learner/agent), timestamps, and a quick add-comment text input"
  - "The drawer can be toggled via an intuitive button on the rehearsal/exam interface and a keyboard shortcut (e.g. 'C'), without resetting the run clock or interrupting active recall flow"
---

## Context
During rehearsal runs and exams, learners frequently want to consult notes, personal mnemonics, or AI-generated explanations attached to the active item without leaving the rehearsal flow or opening the full item editor notebook.

In STORY-0135, the core item comment system was introduced with YAML front-matter persistence and an Item Details notebook tab. As affirmed in user requirements, comments should now be inspectable and appendable directly within active rehearsal and exam runs via an optional, expandable slide-out drawer.

## Description
Deliver an in-rehearsal and in-exam expandable comments drawer in `crates/studyfrog-ui`:

1. **Rehearsal & Exam Comments Drawer (`crates/studyfrog-ui/src/view/run.rs`, `run_card.rs`, `exam_card.rs`):**
   - Add a toggle button (e.g. "Comments" with comment count badge if present) in the top-right or action bar of the rehearsal card view.
   - Support a dedicated keyboard shortcut (e.g. `C` or `Shift+C`) to toggle the drawer open or closed.
   - Render the drawer as an overlaid side panel or bottom collapsible drawer showing:
     - Chronological comment cards with author tags and formatted timestamps.
     - Markdown rendering for comment content.
     - Quick inline text input and "Add" button to attach reflections or study hints during study sessions without interrupting the rehearsal flow.
2. **State & Rehearsal Flow Preservation:**
   - Opening or closing the comments drawer does not reset reading timers, alter card queue order, or trigger accidental reveal/rating actions.

## Acceptance Criteria
- [x] Rehearsal and exam mode views provide an optional expandable drawer or slide-out panel allowing learners to view and add comments for the current card on screen
- [x] The comments drawer displays chronological comments, author tags (learner/agent), timestamps, and a quick add-comment text input
- [x] The drawer can be toggled via an intuitive button on the rehearsal/exam interface and a keyboard shortcut (e.g. 'C'), without resetting the run clock or interrupting active recall flow

## Design Notes
Touchpoints across crates:
- `crates/studyfrog-ui`: `view/run.rs`, `view/run_card.rs`, `view/exam_card.rs`, `setup.rs`, `msg.rs`.
- Guardrails: Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- STORY-0135 (Comments for items).
- EPIC-039 (Universal item linking between stacks, cards, questions, answers, and notes).

## Open Questions
- None.
