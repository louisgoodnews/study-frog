---
id: TASK-0314
type: task
title: "Prevent rating pills and chips from crushing vertically at the right edge of wrapped rows (Diary & kit::pill)"
status: backlog
epic: EPIC-073
parents: ['STORY-0250']
estimate: 1
labels: [m14, ui]
acceptance_criteria:
  - 'Rating pills in the Diary list and calendar views wrap cleanly as whole single-line chips onto the next row instead of crushing vertically into 2-character-wide columns at the right edge'
  - 'kit::pill / kit::figure_pill renders its label with TextWrapMode::Extend (no internal mid-word wrapping) so horizontal_wrapped layouts move overflowing pills cleanly to the next line across all screens'
  - 'Headless UI test verifies Diary entry cards with 12+ rating pills wrap cleanly without tall crushed pills'
---

## Context
Reported by the user with a screenshot of the **Diary** list view (`Diary: 2026-11-26` with 13 rating pills): the rightmost pill (`understanding: 1/5`) gets squeezed into the narrow leftover space on the right edge and breaks vertically into a tall 7-line column (`un / de / rst / an / di / ng: / 1/5`), leaving large empty vertical space below the first row of pills.

## Description
Fix chip/pill wrapping in `crates/studyfrog-ui/src/view/kit.rs` (`figure_pill` / `pill`) and verify `crates/studyfrog-ui/src/view/diary/list.rs` & `calendar.rs` so pills stay single-line and wrap cleanly onto subsequent rows inside `ui.horizontal_wrapped`:
1. In `kit::figure_pill`, render the pill text with `egui::Label::new(...).extend()` (or `TextWrapMode::Extend`) so egui measures the pill's full single-line width and wraps the entire pill to the next row when it does not fit on the current line.
2. Check Diary list/calendar card header alignment (right-aligned `Edit` / `Open in Editor` buttons vs wrapped rating pills) to ensure crisp visual alignment on the right side.
3. Add a headless UI regression test in `crates/studyfrog-ui/tests/diary_ui.rs` with 13 rating pills asserting every pill stays single-line height (`<= 32 px`).

## Acceptance Criteria
- [ ] Rating pills in the Diary list and calendar views wrap cleanly as whole single-line chips onto the next row instead of crushing vertically into 2-character-wide columns at the right edge
- [ ] `kit::pill` / `kit::figure_pill` renders its label with `TextWrapMode::Extend` (no internal mid-word wrapping) so `horizontal_wrapped` layouts move overflowing pills cleanly to the next line across all screens
- [ ] Headless UI test verifies Diary entry cards with 12+ rating pills wrap cleanly without tall crushed pills

## Design Notes
- Affected files:
  - `crates/studyfrog-ui/src/view/kit.rs` (`figure_pill`)
  - `crates/studyfrog-ui/src/view/diary/list.rs`
  - `crates/studyfrog-ui/src/view/diary/calendar.rs`

## Dependencies
- `STORY-0250`

## Open Questions
None.
