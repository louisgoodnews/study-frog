---
id: BUG-0036
type: bug
title: "Reminder toasts cover the Day view's '+ Add session' button"
status: backlog
estimate: 1
labels: [m14, calendar, ui]
acceptance_criteria:
  - "With two or more reminder toasts showing, the Day view's '+ Add session' button and the Day view's header stay clickable"
  - "Toasts sit where they do not cover the page's primary action, or the page reserves room for them"
---

## Context
Found while auditing STORY-0166 in the running app (2026-11-22). After the 24 h and 2 h homework reminders fired, two toasts stacked at the top right of the window and covered the blue "+ Add session" button on the Day view (`/tmp/claude-0/audit/g5/0166-day-kurzquiz.png`). The Console's description line was also covered by a single toast (`/tmp/claude-0/audit/g5/0150-console.png`). Toasts close only on their X, so the button stayed hidden until they were dismissed.

## Description
Keep the toast stack from covering the primary action of the screen: move the stack to a corner that does not hold the header action, or limit the number of visible toasts and let the page reserve their space. Check with two reminders on the Day view at 1280x800.

## Acceptance Criteria
- [ ] With two or more reminder toasts showing, the Day view's '+ Add session' button and the Day view's header stay clickable
- [ ] Toasts sit where they do not cover the page's primary action, or the page reserves room for them

## Design Notes
- Touchpoints: the toast layer in `crates/studyfrog-ui/src/` (notice and toast rendering), `view/planner` (Day view header).
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0166

## Open Questions
- None.
