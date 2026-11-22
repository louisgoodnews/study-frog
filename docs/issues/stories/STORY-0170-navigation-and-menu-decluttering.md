---
id: STORY-0170
type: story
title: "Menu De-Cluttering, Visual Spacing, and Progressive Disclosure"
status: in_progress
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

## Open Questions
None.
