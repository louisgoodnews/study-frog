---
id: BUG-0023
type: bug
title: "The rehearse setup's Case study button ignores the ticked stack and says to open a stack first"
status: done
estimate: 2
labels: [m14, rehearse, ui]
acceptance_criteria:
  - "With a stack ticked in the rehearse setup's What step, Case study opens that stack's cases, or the setup says which stack it uses"
  - "With no stack ticked and none open in the library, the message tells the student to tick a stack or open one"
---

## Context

Found while checking STORY-0181 in the running app (2026-11-22). In the rehearse setup's What step the student ticks a stack (the "Diag 3 items" row is ticked), then presses "Case study". The case view says "Open a stack first, then its case studies." because `setup::case::open` reads the library's selection (`state.browser.selection`), not the ticked stacks.

Screenshots: `/tmp/claude-0/audit/g3/08a-setup.png` (the message, shown after the setup was reopened with Diag ticked) and `/tmp/claude-0/audit/g3/08b-setup.png` (the same setup with Diag ticked, before the case view was reopened). The case view worked only after `stack open diag` in the library.

Reproduction: `studyfrog ctl "vault open <folder with a case study>"`, `ctl "nav rehearse"`, tick the stack in the Stacks box, press "Case study". The message appears. Opening the stack in the library first (`ctl "stack open <folder>"`) makes the case view work.

**Fix (2026-11-22).** The case view reads the stack ticked in the What step, or the stack open in the library when none is ticked (`crates/studyfrog-ui/src/setup/stack_source.rs`). Test: `studyfrog-ui/tests/case_study_ui.rs` `the_case_study_reads_the_stack_ticked_in_the_setup_not_the_open_one`; with nothing ticked or open the message names both ways to choose. Running app (Pair ticked): the case view opens for the ticked stack instead of "Open a stack first" (`/tmp/claude-0/fix1/0023-case-pair.png`). That stack has no cases, so the screen itself does not name it; the test checks the stack read.

## Description

Make the case view use the stack ticked in the setup (or the one open in the library when none is ticked), in `crates/studyfrog-ui/src/setup/case.rs` (`open`). If the setup has no stack ticked and none is open, keep the message and name both ways to choose a stack.

## Acceptance Criteria

None.

## Design Notes

- Guardrails: P4 (nesting < 3) and P6 (files <= 250 lines).

## Dependencies

- STORY-0181

## Open Questions

- None.
