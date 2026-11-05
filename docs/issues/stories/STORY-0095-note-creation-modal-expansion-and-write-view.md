---
id: STORY-0095
type: story
title: Note creation modal full-window expansion and default write view
status: done
epic: EPIC-031
parents: [EPIC-031]
estimate: 3
labels: [m11, ui]
acceptance_criteria:
  - "Selecting 'note' as the item kind in the create modal automatically expands the modal to occupy the full window width and height"
  - "The note Markdown editor in the create modal defaults to write mode rather than split preview mode"
  - "Users can still resize the expanded note modal via the drag handle, shrink it using the header button, or switch to other creatable item types"
---

## Context
When drafting study notes in StudyFrog, users trigger the creation modal (`modal_create.rs`). Unlike flashcards or questions—which have short prompts and answers—notes are comprehensive markdown documents requiring significant screen area for outlining, writing, and formatting.

Currently, the creation modal imposes two ergonomic limitations on note creation:
1. `crates/studyfrog-ui/src/view/modal_size.rs` defines `WIDE = 1000.0`, which constrains modal width on wide displays and limits modal height to `content_rect().height() - 200.0`. It does not fill the screen.
2. `crates/studyfrog-ui/src/view/modal_create.rs` defaults notes to `Mode::Split` (split editor and preview), cutting the horizontal writing width in half and cramping text entry during note drafting.

## Description
Optimize the note creation experience in the create modal:
1. Automatic Full-Window Expansion:
   - When the created item kind is set to `NewKind::Note`, the modal dimensions automatically expand to fill the entire window area (e.g. `widest(ctx)` and full available vertical height minus minimal padding), unless the user has manually set a custom width override.
   - When switching back to Card, Question, or Stack, the modal returns to standard compact dimensions.
2. Default Write View:
   - Set the default editor view mode for notes to `Mode::Write` instead of `Mode::Split`, dedicating maximum visual focus and horizontal room to the authoring editor.
   - Users can toggle to split or preview mode at any time using existing mode switch controls.
3. Resizing Flexibility:
   - Retain full interactive control: users can manually drag the bottom-right resize grip, click the shrink dialog button to toggle between full-window and normal size, or switch item types seamlessly.

## Acceptance Criteria
- [x] Selecting 'note' as the item kind in the create modal automatically expands the modal to occupy the full window width and height
- [x] The note Markdown editor in the create modal defaults to write mode rather than split preview mode
- [x] Users can still resize the expanded note modal via the drag handle, shrink it using the header button, or switch to other creatable item types

## Design Notes
- Files to touch:
  - `crates/studyfrog-ui/src/view/modal_create.rs`: Set `Mode::Write` as initial mode for notes in `editor_widget(...)`.
  - `crates/studyfrog-ui/src/view/modal_size.rs`: Calculate full window dimensions for note kind when unconstrained by user temp overrides. Ensure shrink button toggles between full size and compact size.
- Maintain conformance to line count limits (keep `.rs` files ≤ 250 lines).
- Verify with `crates/studyfrog-ui/tests/create_dialog_layout.rs` and manual UI inspection.

## Dependencies
EPIC-031.

## Open Questions
None.
