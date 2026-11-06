---
id: STORY-0109
type: story
title: Image occlusion cards for diagrams, slides, and PDF figures
status: backlog
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
- [ ] Users can create an Image Occlusion card from any vault image or extracted PDF figure by drawing rectangular occlusion masks and assigning each mask a label/answer
- [ ] Occlusion masks are stored non-destructively in plain-text Markdown/front-matter using normalized [0.0, 1.0] coordinates without modifying the source image file
- [ ] Rehearsal supports both 'Hide All, Guess One' and 'Hide One, Guess One' modes, revealing the highlighted target region upon flip or typed answer submission

## Design Notes
- Touchpoints:
  - `crates/studyfrog-core/src/doc.rs` & `crates/studyfrog-md`: occlusion mask data structures and round-trip serialization.
  - `crates/studyfrog-ui/src/view/editor.rs`: mask drawing/editing overlay canvas.
  - `crates/studyfrog-ui/src/view/rehearse.rs`: overlay rendering during rehearsal.
- Strictly enforce P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-035.

## Open Questions
None.
