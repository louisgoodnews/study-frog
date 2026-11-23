---
id: EPIC-057
type: epic
title: "UI/UX Streamlining, Universal Import Hub, and Cognitive De-Cluttering"
status: review
parents: []
labels: [m14, ui, ux]
acceptance_criteria:
  - 'Unified single Import action replaces fragmented card, stack, note, and question buttons'
  - 'Systematic contextual tooltips and hover hints across all navigation icons, buttons, and tools'
  - 'De-cluttered menu bars with progressive disclosure, reducing cognitive overload for new learners'
  - 'Lightweight, modern visual rhythm maintaining deep power features under intuitive secondary disclosures'
---

## Context
StudyFrog has accumulated numerous specialized capabilities across 22 crates, but interface density has outpaced intuitive usability. Overpopulated menus, redundant buttons (e.g. separate card import vs stack/note import), and a lack of hover hints make the application feel overly complicated rather than light and supportive.

## Description
Conduct a comprehensive user experience refinement focused on simplicity, clarity, and delight:
1. **Unified Universal Import**: Replace fragmented, disparate import buttons with a single clear "Import" action featuring intelligent content auto-detection.
2. **Contextual Tooltips & Guidance**: Add helpful hover tooltips across the navigation rail, action icons, headers, and modal actions, including keyboard shortcut badges.
3. **Menu & Action De-Cluttering**: Clean up crowded headers and menus by grouping secondary tools into progressive disclosures (`...` overflow menus) and increasing breathing room.
4. **Lightweight Persona Alignment**: Rebalance information hierarchy so the app feels approachable and focused for daily study while keeping advanced features effortlessly discoverable.

## Acceptance Criteria
- [x] Unified single Import action replaces fragmented card, stack, note, and question buttons
- [x] Systematic contextual tooltips and hover hints across all navigation icons, buttons, and tools
- [x] De-cluttered menu bars with progressive disclosure, reducing cognitive overload for new learners
- [x] Lightweight, modern visual rhythm maintaining deep power features under intuitive secondary disclosures

## Design Notes
Implement tooltips using `egui::Response::on_hover_text` and `on_hover_ui`. Strictly use palette color tokens from `palette_of(ui)` (no raw RGB). Follow the established rule of visual halves for spacing and padding.

## Dependencies
None.

## Open Questions
None.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): criterion 1 MET: one Import... entry (items toolbar More..., stack page More actions); no separate import buttons seen (/tmp/claude-0/audit/g5/0168-more-menu.png). Criterion 2 MET: all header icon buttons, the rehearsal rating buttons and the Show answer button have tooltips with shortcuts (/tmp/claude-0/audit/g5/0169-header-tooltips.png, 0169-combo.png). Criterion 3 NOT MET: the stack page, item list and item editor are decluttered, but the PDF viewer's header still shows ten controls (/tmp/claude-0/audit/g5/0170-pdf.png); see STORY-0170. Left unticked. Criterion 4 NOT CHECKABLE: whether the look is lightweight is a judgement; the spacing and type scale are checked in STORY-0170 (/tmp/claude-0/audit/g5/0170-editor.png).

## Verification (2026-11-22)

- Criterion 3 stays unticked: NOT MET while the PDF viewer header shows ten buttons (STORY-0170 criterion 1; `/tmp/claude-0/verifyA/16-pdf-1280-empty.png`).
- Criterion 4 NOT CHECKABLE: "lightweight, modern visual rhythm" is a judgement; no check made here.
- Criteria 1 and 2 were not re-checked in this round. Status stays review.

## Review (2026-11-22)

Review (2026-11-22): criterion 1 stays MET: one Import entry replaces the duplicate import buttons. Criterion 2 unticked: the navigation rail's text items (Home, Calendar, Homework, Board and the rest) show no hover text after 3.5 s (/tmp/claude-0/review-a/hover-home-long.png, /tmp/claude-0/review-a/hover-calendar.png), and the help tooltip registry lists none of them. The top bar icons and "+ Create" do have explanations (/tmp/claude-0/review-a/hover-create.png). Criterion 3 NOT MET (PDF viewer header, STORY-0170). Criterion 4 NOT CHECKABLE: "lightweight, modern visual rhythm" is a judgement. Status stays review.

## Verification (2026-11-23)

- Criterion 3 ticked: the PDF viewer header now has four buttons (Back to library, Previous, Next, More…) with zoom, hit navigation, copy, arrange and the panels in More (STORY-0170 criterion 1; `/tmp/claude-0/pdfhdr/after-1280-hit.png`, `after-1280-more-hit.png`). The stack page, item list and item editor were decluttered earlier (2026-11-21 and audit). The owner should confirm the reading of "core" recorded in STORY-0170.
- Criterion 2 stays unticked (review of 2026-11-22: the navigation rail's text items show no hover text). Criterion 4 stays unticked: "lightweight, modern visual rhythm" is a judgement, not checked here.
- Status stays review: criteria 2 and 4 are open.

## Verification (2026-11-23)

- Criterion 2 ticked: the navigation rail's text items now have explanations (STORY-0189; `/tmp/claude-0/g1/shots/after1-hover-calendar-1280.png`), the rehearsal action tray's Comments toggle and the transfer dialog's tabs too, and the guard test covers the rail's selectable buttons and the link buttons. The header icons and "+ Create" were already explained (review of 2026-11-22). The rest of the buttons are covered by `tests/text_button_tooltips.rs`, which fails on any new unexplained button.
- Criterion 4 stays unticked: "lightweight, modern visual rhythm" is a judgement, not checked here.
- Child stories: STORY-0168 and STORY-0170 stay in review (STORY-0170 is owner-decided and was not changed); STORY-0169 is done; STORY-0189 is done. The epic stays in review, so it is not set to done.


Decision (2026-11-23): criterion 4 ticked on the owner's confirmation that no criterion is unmet. EPIC-057 stays review until STORY-0170 is confirmed.
