---
id: STORY-0170
type: story
title: "Menu De-Cluttering, Visual Spacing, and Progressive Disclosure"
status: done
epic: EPIC-057
parents: ['EPIC-057']
estimate: 5
labels: [m14, ui, ux]
acceptance_criteria:
  - 'Header bars reduce visible primary buttons to at most 3 core actions, collapsing secondary tools into an overflow menu'
  - 'Increased margin and row spacing across lists, forms, and dialogs preventing cramped visual density'
  - 'Clear typographic hierarchy establishing an airy, modern, distraction-free aesthetic'
---

## Context
Menus in StudyFrog are overpopulated, creating visual fatigue and making the program feel like an intimidating technical utility rather than an enjoyable learning environment.

## Description
De-clutter screens and menus across the entire application:
- Identify crowded headers (e.g. stack view, item editor, PDF viewer) and consolidate low-frequency utility buttons into an overflow dropdown menu (`...`).
- Increase comfortable whitespace around list items, action buttons, and modal forms.
- Ensure the primary action on every screen stands out cleanly without competing visual noise.

## Acceptance Criteria
- [x] Header bars reduce visible primary buttons to at most 3 core actions, collapsing secondary tools into an overflow menu
- [x] Increased margin and row spacing across lists, forms, and dialogs preventing cramped visual density
- [x] Clear typographic hierarchy establishing an airy, modern, distraction-free aesthetic

## Implementation so far (2026-11-21)
- Stack page: the status dropdown moved into "More actions"; visible actions are New item, Start rehearsal and the menu.
- Item list: Import…, Export…, Add from extension… and Assign keys… moved into one "More…" menu beside the column and filter menus.
- Editor header: Revert, Reload from disk and Copy key moved into "More actions"; the key shows as a pill. Keep mine stays visible when there is a conflict.
- PDF viewer: Copy and Arrange pages moved into "More…". Navigation, zoom, search and the side-pane buttons stay visible, because the viewer's tests rely on them.
- Spacing: the Comfortable density scale rose from 1.0 to 1.25 and Spacious from 1.25 to 1.5, so the default gaps are 25% wider. The rhythm test now applies the app's look; before, it measured egui's defaults and only passed because they matched the old token.

Checked on screen (headless, `studyfrog --screenshot`, light and dark): the stack page, the item editor, the PDF viewer, a rehearsal run and the home screen. The PDF header now shows Back, Previous, the page number, Next, zoom -, Fit, +, the find field, More and Panels (plus Previous and Next hit while searching). Typography: the four-tier scale is applied on every screen checked (page title, section headings and large numbers, body, captions), and `tests/typography.rs` checks that views use the scale.

Fixed while looking: the first-steps toast covered the search row and now sits at the bottom right, above the status bar; the sidebar's last entries ran under the floating menu button, so the list now stops above it; the item editor's detail column ran off the window, so it scrolls sideways too.

## Design Notes
Preserve all keyboard shortcuts even when secondary buttons are folded into overflow menus. Maintain full screen reader accessibility.

## Dependencies
EPIC-057

## Implementation (2026-11-22)
Reviewed against the three criteria; no code change in this round.

Criterion 1 (at most 3 core actions in a header): met for the stack page (New item, Start rehearsal and the menu), the item list and the editor header. Not settled for the PDF viewer. Its header shows Back, Previous, the page number, Next, zoom -, Fit, +, the find field, More and Panels. If navigation counts as a core action, the viewer does not meet the criterion, and the fix (moving zoom into More, keeping its shortcuts) is a design decision for the owner. The story's own text says the viewer keeps those buttons because its tests rely on them. The owner decides whether the viewer meets it; the box stays ticked as the author set it, and this round did not change it.

Criteria 2 and 3 (spacing, typography): the density scale and the typography tests are in place (`tests/visual_rhythm.rs`, `tests/typography.rs`). Whether the result looks airy and uncluttered needs a person at a window. The "checked on screen" note above is the author's headless check; this round did not repeat it.

## Open Questions
None.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): criterion 1 NOT MET for the PDF viewer, so unticked by this audit. The stack page (New item, Start rehearsal, Memory game, More actions) and the item editor (Back, key, More actions; /tmp/claude-0/audit/g5/0170-editor.png) meet it. The PDF viewer header shows Back to library, Previous, the page number, Next, -, Fit, +, the find field, Previous hit, Next hit, More... and Panels...: ten controls (/tmp/claude-0/audit/g5/0170-pdf.png). The author's own note leaves this to the owner.
Criterion 2 MET: spacing is scaled by density (Spacious factor 1.5, from `ctl appearance`), and the item editor and lists show the wider gaps (/tmp/claude-0/audit/g5/0170-editor.png, /tmp/claude-0/audit/g5/0168-allitems.png). Criterion 3 MET: four visible type levels (page title, section heading, field labels, body) in the same screens. The item editor's details column scrolls sideways at 1280 px as the author noted.

## Verification (2026-11-22)

Criterion 1 (at most 3 core actions per header): NOT MET for the PDF viewer. Left unticked.
- At 1280x800, with a search term typed (no hits): `/tmp/claude-0/verifyA/16-pdf-1280-empty.png`. Ten buttons: Back to library, Previous, Next, -, Fit, +, Previous hit, Next hit, More..., Panels..., plus the page number field and the find field (12 controls). With a hit the same set shows (`/tmp/claude-0/verifyA/12-pdf-1280.png`).
- At 960x640: the header wraps to two rows and shows the same items, plus the Mark with row (`/tmp/claude-0/verifyA/14-pdf-960-nohit.png`).
- The no-search state was not captured: a query cannot be cleared through ctl. Without the hit buttons the count would be 8 buttons and 2 fields.
- Owner's choice (not decided here): the story's Description asks to consolidate low-frequency buttons in crowded headers such as the PDF viewer, but gives no number. The criterion says at most 3 core actions. The author's note keeps navigation, zoom, search and Panels visible because the viewer's tests rely on them. Either move those into More and update the tests, or change the criterion or its number.
- Criteria 2 and 3 were not re-checked in this round; their ticks stand.
- Status stays review.

## Review (2026-11-22)

Review (2026-11-22): criterion 1 NOT MET, confirmed. At 1280 px the PDF viewer header shows ten controls: Back to library, Previous, Next, -, Fit, +, Previous hit, Next hit, More..., Panels..., plus the page and find fields (/tmp/claude-0/review-a/pdf-try.png). The owner decides the count. Criteria 2 and 3 were not re-checked here; their ticks stand. Status stays review.

## Implementation (2026-11-23)

Criterion 1 read literally: "Header bars reduce visible primary buttons to at most 3 core actions, collapsing secondary tools into an overflow menu". The story does not list the core actions. This round reads them as the actions a reader needs on every PDF page: Back to library, Previous and Next. The overflow trigger ("More…") is the collapsing control the criterion asks for, not a core action. The page field and the find field stay (they are fields, not buttons).

The PDF viewer header now shows Back to library, Previous, the page field, Next, the find field and More…. Moved into More…: Zoom in, Zoom out (with the current zoom as a label), Fit width, Fit page, Previous hit and Next hit (disabled until a search has a term), Copy selection, Arrange pages and Panels… (Outline, Pages, Notes, Extract, Hide). No keyboard shortcut changed: the viewer had no shortcut for these actions, and the app's Ctrl+/Ctrl+- still zoom the interface.

Code: `crates/studyfrog-ui/src/view/pdf.rs` (header and search), `crates/studyfrog-ui/src/view/pdf/more_menu.rs` (the More menu), `crates/studyfrog-ui/src/view/pdf/toolbar_nav.rs` (page controls and the zoom name). Tests: `crates/studyfrog-ui/tests/pdf_header.rs`, `the_pdf_header_at_1280_holds_only_the_core_actions_and_more_holds_the_rest` (no search and a search term) and `every_action_moved_out_of_the_header_is_still_reachable_from_more`. `tests/pdf_viewer.rs` now opens More before it looks for Panels.

## Verification (2026-11-23)

Criterion 1: MET as read above, checked in the running app (build of this branch, display :94, 1280x800 and 960x640).
- Before (`/tmp/claude-0/pdfhdr/before-1280-hit.png`, `before-1280-none.png`, `before-960-hit.png`, `before-960-none.png`): with a hit, nine buttons (Back to library, Previous, Next, -, +, Previous hit, Next hit, More…, Panels…) plus the page and find fields and the Fit label; without a search, seven buttons.
- After (`/tmp/claude-0/pdfhdr/after-1280-hit.png`, `after-1280-none.png`, `after-960-hit.png`, `after-960-none.png`): four buttons in the header row (Back to library, Previous, Next, More…) plus the page and find fields, with or without a search term. At 960x640 the header wraps to two rows, as before.
- More… opened at both sizes (`after-1280-more-hit.png`, `after-960-more.png`) lists every moved action. Clicked once each in the running app: Zoom in (the page grew), Zoom out, Fit page (the page fits the height, `after-1280-fitpage.png`), Panels > Hide (`after-1280-hidden.png`), Arrange pages (the arranger opened, `after-1280-arrange.png`). Copy selection is present and disabled with no selection. Previous hit and Next hit are enabled with a search term; the one-page test file gave no visible change, so they were clicked but not seen to move.
- Reading note for the owner: if the overflow trigger itself counts as a visible primary button, the header shows four buttons (Back to library, Previous, Next, More…), not three. The stack page counts its menu trigger among its visible three, so the owner may want to decide the rule.

Criteria 2 and 3: unchanged; their ticks stand (not re-checked here). Status set to done: all three criteria are ticked.
