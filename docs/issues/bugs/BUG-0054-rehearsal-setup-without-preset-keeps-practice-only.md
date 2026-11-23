---
id: BUG-0054
type: bug
title: "A rehearsal setup without a preset keeps the previous practice preset, so the run writes no reviews"
status: done
estimate: 2
labels: [m14, rehearsal, ui]
acceptance_criteria:
  - "A setup without --preset (control) or without a chosen preset (app) starts from Spaced, and a run from it writes review rows"
  - "A practice preset (all, shuffled, hardest, stalest) chosen in the same setup still runs practice-only"
---

## Context
A fresh vault's reports showed 0 reviews after a rehearsal run. The run had been set up with `rehearse setup --preset all --stack x` earlier in the session; the next `rehearse setup --stack x` did not name a preset, so the form kept the practice-only order and effect of `all`. The run was practice-only, so it wrote no review rows.

## Description
The owner decided that a setup without a preset goes back to Spaced. Spaced runs update the schedule and write reviews; the practice presets (`all`, `shuffled`, `hardest`, `stalest`) do not, as documented in [rehearsal-run](../../specs/rehearsal-run.md). A new setup, in the control protocol or on the rehearse screen, must start from Spaced unless the user picks another preset in that same setup. Stacks, kinds and tags chosen before still carry over.

## Acceptance Criteria
- [x] A setup without `--preset` (control) or without a chosen preset (app) starts from Spaced, and a run from it writes review rows
- [x] A practice preset chosen in the same setup still runs practice-only

## Design Notes
- The control answers `rehearse setup` with the preset the form now holds (`"preset"` in the rehearse snapshot; `null` when the user changed a field away from a preset).
- Cause, control path: `control_setup::setup_msgs` sends a preset edit only when `--preset` is given.
- Cause, app path: opening the rehearse screen from another screen keeps the form's preset (`nav::screen_effects` only refreshes the preview).
- Fix at the cause: a setup always starts with a preset edit (the given one, else Spaced); the rehearse screen resets the preset when it is entered from another screen. Mode, stacks, kinds and tags are kept as before.
- Guardrails: P1, P2, P4, P6 (file size).

## Dependencies
- docs/specs/rehearsal-run.md (defaults for practice presets)

## Verification (2026-11-23)

Fix (2026-11-23): `setup::start_setup` resets the form to Spaced when the rehearse screen is entered from another screen (`nav::navigate`), keeping stacks, kinds, tags and mode; `control_setup::setup_msgs` always sends a preset edit, Spaced when `--preset` is not given; the rehearse snapshot reports `preset` through `setup::preset_of` (the preset the form matches, `null` otherwise). Tests (fail before, pass after): `studyfrog-ui/tests/control_setup_default.rs` (`a_setup_without_a_preset_starts_spaced_after_a_practice_preset`, and a run from the defaulted setup writes reviews while the `all` run writes none), `studyfrog-ui/tests/setup_default_form.rs` (a new rehearse setup starts Spaced; a practice preset chosen in the same setup stays PracticeOnly). The related existing tests (setup steps, control session, control events, control state, control exam, control parse, planning) still pass.

App check, the running app on display :98 (vault with stack French and 3 cards). Before (pre-fix binary): `rehearse setup --preset all --stack french` run: `order_name` sequential, `effect` practice_only, 0 review rows; then `rehearse setup --stack french` (no preset) run: `order_name` sequential, `effect` practice_only, still 0 review rows (`/tmp/claude-0/preset/vault/.studyfrog/index.sqlite`, `run` table). Screenshot `/tmp/claude-0/preset/reports-before.png` shows no reviews. After (fixed binary): the same `all` run writes 0 reviews; `rehearse setup --stack french` answers `"preset": "spaced"`, the run is `order_name` due with `effect` update_schedule and writes 3 review rows; `nav home` then `nav rehearse` after an `all` setup answers `spaced`; `rehearse setup --preset all` answers `all`. Screenshot `/tmp/claude-0/preset/reports-after.png` shows 3 reviews.

## Open Questions
- None.
