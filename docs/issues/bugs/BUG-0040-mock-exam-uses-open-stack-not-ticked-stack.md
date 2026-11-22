---
id: BUG-0040
type: bug
title: "Mock exam draws from the open stack, not the stack ticked in the rehearse setup's What step"
status: done
estimate: 2
labels: [m14, rehearse, ui]
acceptance_criteria:
  - "With a stack ticked in the What step, Mock exam makes the exam from that stack's notes, whichever stack is open in the library"
  - "When the open stack and the ticked stack differ, the exam view names the stack it used"
---

## Context
Found while auditing STORY-0186 in the running app (2026-11-22). With the stack "Statistik" ticked in the rehearse setup's What step, "Mock exam" first made the exam from the sub-stack `statistik/statistik` (questions on "Hauptstadt" and "Wurzel", cards from a CSV import), because that stack was open in the library. The exam title read "Mock exam: statistik/statistik". After the library was opened on "Statistik", the same button used the notes of Statistik. Compare BUG-0023 (Case study ignores the ticked stack) and BUG-0024 (Memory game setup ignores ticked stacks): the same rule is not applied to the mock exam.

Screenshots: `/tmp/claude-0/audit/g5/0186-mock-notes.png` (sub-stack used), `/tmp/claude-0/audit/g5/0186-mock-notes2.png` (after opening Statistik).

**Fix (2026-11-22).** The mock exam is made from the ticked stack, or else the open stack (`setup/stack_source.rs`, `setup/mock_exam/open.rs`); the exam title names the stack. Test: `studyfrog-ui/tests/mock_exam_ui.rs` `the_exam_is_made_from_the_stack_ticked_in_the_setup_and_names_it`. Running app (Pair ticked, Geo open): "Mock exam: pair" (`/tmp/claude-0/fix1/0040-mock-pair.png`).

## Description
Build the mock exam from the stacks ticked in the What step, as the setup says, and fall back to the open stack only when none is ticked. Show the stack name in the exam title.

## Acceptance Criteria
- [x] With a stack ticked in the What step, Mock exam makes the exam from that stack's notes, whichever stack is open in the library
- [x] When the open stack and the ticked stack differ, the exam view names the stack it used

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/setup/mock_exam.rs`, `crates/studyfrog-ui/src/exec/mock.rs`.
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0186
- BUG-0023
- BUG-0024

## Open Questions
- None.
