---
id: TASK-0315
type: task
title: "Highlight active Rehearse setup preset and add ergonomic, enforced run item-limit controls"
status: backlog
epic: EPIC-073
parents: ['STORY-0251']
estimate: 2
labels: [m14, ui, rehearse]
acceptance_criteria:
  - 'In setup_how.rs, the 5 preset buttons render as highlighted selectable pills reflecting the active preset (derived from SetupForm state or tracked active preset)'
  - 'The Maximum number of items control provides quick-pick limit buttons (No limit, 10, 20, 30, 50) alongside the custom numeric input, and displays a clear hint if the entered value is not a positive integer'
  - 'Verify and test that the configured limit caps both the Step 2/Step 3 preview count and the generated run queue in studyfrog-service / studyfrog-rehearsal'
  - 'Headless UI tests in crates/studyfrog-ui/tests/ verify preset highlight state and item-limit selection'
---

## Context
Part of `STORY-0251`. In `crates/studyfrog-ui/src/view/setup_how.rs:52-62`, the preset buttons use `ui.button(label)` so the selected mode is never highlighted, and the `Maximum number of items` input has no quick-pick options or inline validation feedback.

## Description
1. Track or derive the currently active `Preset` from `SetupForm` and render each preset in `setup_how.rs` using `ui.selectable_label(is_active, label)` with hover explanation.
2. Enhance the `Maximum number of items` section in `setup_how.rs` with quick-pick limit chips (`No limit`, `10`, `20`, `30`, `50`) plus the custom numeric `TextEdit`, and verify `form.limit` cleanly caps both `preview_queue` and `new_run`.
3. Add headless UI and service tests confirming preset highlighting and item-limit capping.

## Acceptance Criteria
- [ ] In `setup_how.rs`, the 5 preset buttons render as highlighted selectable pills reflecting the active preset (derived from `SetupForm` state or tracked active preset)
- [ ] The `Maximum number of items` control provides quick-pick limit buttons (`No limit`, `10`, `20`, `30`, `50`) alongside the custom numeric input, and displays a clear hint if the entered value is not a positive integer
- [ ] Verify and test that the configured limit caps both the Step 2/Step 3 preview count and the generated run queue in `studyfrog-service` / `studyfrog-rehearsal`
- [ ] Headless UI tests in `crates/studyfrog-ui/tests/` verify preset highlight state and item-limit selection

## Design Notes
- Affected files:
  - `crates/studyfrog-ui/src/view/setup_how.rs`
  - `crates/studyfrog-ui/src/setup/mod.rs` (or `setup/form.rs`)
  - `crates/studyfrog-ui/tests/setup_steps.rs` / `setup.rs`

## Dependencies
- `STORY-0251`

## Open Questions
None.
