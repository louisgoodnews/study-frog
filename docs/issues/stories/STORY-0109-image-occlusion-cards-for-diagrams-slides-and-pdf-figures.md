---
id: STORY-0109
type: story
title: "Image occlusion cards for diagrams, slides, and PDF figures"
status: done
epic: EPIC-035
parents: [EPIC-035]
estimate: 8
labels: [m12, core, md, pdf, rehearsal, ui]
acceptance_criteria:
  - "Users can create an Image Occlusion card from any vault image or extracted PDF figure by drawing rectangular occlusion masks and assigning each mask a label/answer"
  - "Occlusion masks are stored non-destructively in plain-text Markdown/front-matter using normalized [0.0, 1.0] coordinates without modifying the source image file"
  - "Rehearsal supports both 'Hide All, Guess One' and 'Hide One, Guess One' modes, revealing the highlighted target region upon flip or typed answer submission"
---

## Context
Visual diagrams—such as brain anatomy, statistical path diagrams, UML architectures, or lecture slide charts—are tedious or impossible to convert into text-only flashcards. **Image Occlusion** lets the learner draw rectangular masks over labels on an image and rehearse each masked region individually. Storing the masks as normalized coordinates (`x, y, w, h` in `[0.0, 1.0]`) inside the `.md` card file keeps the vault 100% plain-text and non-destructive.

## Description
Implement Image Occlusion card creation, plain-text serialization, and rehearsal rendering:
1. **Plain-Text Markdown Representation (`crates/studyfrog-core`, `crates/studyfrog-md`):**
   - Represent occlusion regions on a card referencing an image (`![alt](assets/diagram.png)`) via structured mask entries (e.g., `id`, normalized `x`, `y`, `w`, `h` in `[0.0, 1.0]`, optional `label` text for typed-answer comparison, and `group`).
   - Support occlusion mode setting: `HideAllGuessOne` (all masks opaque, active mask highlighted in accent color) or `HideOneGuessOne` (only the active mask opaque).
2. **Interactive Mask Editor (`crates/studyfrog-ui`):**
   - In the card editor (and directly from the PDF viewer's region extraction tool), provide an interactive canvas to drag-draw, move, resize, and delete rectangular masks over the image and optionally type the occluded text label for semantic/typed grading.
3. **Rehearsal Integration:**
   - Render the image with opaque overlay rectangles during the prompt phase, highlighting the target rectangle with a distinct border/color.
   - Upon reveal (or after submitting a written answer matching the mask's `label`), hide the target mask so the underlying diagram label is visible.

## Acceptance Criteria
- [x] Users can create an Image Occlusion card from any vault image or extracted PDF figure by drawing rectangular occlusion masks and assigning each mask a label/answer
- [x] Occlusion masks are stored non-destructively in plain-text Markdown/front-matter using normalized [0.0, 1.0] coordinates without modifying the source image file
- [x] Rehearsal supports both 'Hide All, Guess One' and 'Hide One, Guess One' modes, revealing the highlighted target region upon flip or typed answer submission

## Design Notes
- Touchpoints:
  - `crates/studyfrog-core/src/doc.rs` & `crates/studyfrog-md`: occlusion mask data structures and round-trip serialization.
  - `crates/studyfrog-ui/src/view/editor.rs`: mask drawing/editing overlay canvas.
  - `crates/studyfrog-ui/src/view/rehearse.rs`: overlay rendering during rehearsal.
- Strictly enforce P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and verified:
- Format `crates/studyfrog-md/src/occlusion.rs`: the `occlusion` front-matter block (picture, mode, masks as
  ten-thousandths of the picture), read, written, replaced and removed; unit tests and a property test that any
  block survives writing and reading and the whole card round-trips losslessly (`crates/studyfrog-md/tests/occlusion.rs`).
- Run content `crates/studyfrog-service/src/content_occlusion.rs`: masks take turns with reviews, prompt without
  the picture, the label as answer and as the typed-answer reference; `create_occlusion_card` makes a card from a
  vault picture without touching the file. Fixture tests `crates/studyfrog-service/tests/occlusion_content.rs`.
- Editor: "Make this an image occlusion card", the canvas (draw by dragging, move, resize by the corner handle,
  select, label, delete, mode, stop), saved through the item form (`EditAction::Occlusion`); geometry in
  `occlusion_ui.rs` with unit tests (`tests/occlusion_ui.rs`); a real drag on a real picture in the egui harness
  draws a mask (`tests/occlusion_views.rs`).
- PDF figures: "Occlusion card from <path>" next to every picture or snapshot saved in the PDF viewer makes the card
  and opens its page (`tests/occlusion_flow.rs`, through the real worker, masks saved to the file).
- Run: the picture with the masks (Hide all / Hide one), uncovered after the flip (`view/occlusion_paint.rs`);
  harness test of hidden and uncovered states, typed answers sent with the label. Screenshots of a run before
  and after the answer and of the card's page with its masks looked at.
- Spec `docs/specs/image-occlusion.md`.

Not built: a separate schedule per mask (masks of one card share its schedule and take turns), polygon masks,
and choosing a picture already in the vault from a file list (the picture is added with the editor's Picture
button or written as Markdown).

## Dependencies
EPIC-035.

## Open Questions
None.
