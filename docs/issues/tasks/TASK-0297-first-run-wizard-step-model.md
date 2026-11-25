---
id: TASK-0297
type: task
title: "Wizard step model: pure state of the optional steps from settings and installed models"
status: done
epic: EPIC-069
parents: ['STORY-0212']
estimate: 3
labels: [m14, ui, tooling]
acceptance_criteria:
  - "The next and previous step, skip, and done are pure functions over the step list and the installed state"
  - "Steps: vault, local model, remote provider, MiniLM, summary; a step already satisfied is shown as done and can still be revisited"
  - "Unit tests cover every transition, including skip and reopen from Settings"
---

## Context
STORY-0212 needs one place that decides which step is next, so the screen and the control protocol agree.

## Description
Add the step model in a pure module (no inherent `impl` blocks, files under 250 lines).

## Acceptance Criteria
- [x] Pure transitions for next, previous, skip and done
- [x] Steps and satisfied state derived from settings and installed models
- [x] Unit tests for every transition

## Design Notes
Keep the step list data-only so the control protocol can print it.

## Dependencies
- STORY-0212.

## Open Questions
None.

## Verification note
Done 2026-11-25. `crates/studyfrog-config/src/wizard.rs` holds the pure step model; `crates/studyfrog-config/tests/wizard.rs` has 10 tests covering every transition, skip, revisit and the status precedence. The step status reads an `Installed` value that the caller fills from settings and the installed models; that wiring belongs to TASK-0299. `scripts/check.sh` passed on the tree before this commit.
