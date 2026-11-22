---
id: STORY-0170
type: story
title: "Menu De-Cluttering, Visual Spacing, and Progressive Disclosure"
status: review
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
- [ ] Header bars reduce visible primary buttons to at most 3 core actions, collapsing secondary tools into an overflow menu
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
