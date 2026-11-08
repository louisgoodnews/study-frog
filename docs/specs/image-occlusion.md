# Image occlusion cards (STORY-0109)

A card that hides parts of a picture and asks for them one at a time. Format and geometry are pure:
`studyfrog_md::occlusion`, `crates/studyfrog-ui/src/occlusion_ui.rs`; run content:
`studyfrog_service::content_occlusion`; window: `view/occlusion_edit.rs`, `view/occlusion_paint.rs`.

## In the file
The picture file is never changed. The card's front shows the picture as ordinary Markdown
(`![brain](assets/brain.png)`, so any Markdown reader shows it); the masks are a front-matter block:

```yaml
occlusion:
  image: "assets/brain.png"
  mode: hide_all            # or hide_one
  masks:
    - "0.1200 0.3400 0.2000 0.0500 Hippocampus"
```

Each mask is `x y w h label`: the left and top edges, width and height as fractions of the picture's
width and height (`0.0000` to `1.0000`, four decimals, stored as ten-thousandths), then an optional label
(what it hides; compared with typed answers). A mask must lie inside the picture and have a size. A
malformed block is kept verbatim in the file but not used. Writing and reading a block gives it back
unchanged (property-tested), and the card's other keys stay as they are.

## Making one
- **From any vault picture:** on a card's page whose front shows a picture, "Make this an image occlusion
  card" adds an empty block for that picture.
- **From a PDF figure:** in the PDF viewer's "Make a note" panel, every saved picture or region snapshot
  has "Occlusion card from <path>": it makes a card "Occlusion: <name>" in the chosen stack (front: the
  picture and "What is hidden under the marked box?", no back yet) and opens its page.
- **Drawing:** under the front, the picture is shown with its masks (numbered, with their labels). Drag on
  the picture to draw a mask; drag a mask to move it; drag the lower right corner of the selected mask to
  resize it (masks stay inside the picture; a box smaller than 1 % of a side is not kept). Click a mask to
  select it, then give it a label or delete it. "Hide all, guess one" / "Hide one, guess one" set the
  mode; "Stop using occlusion" removes the block. Every finished change is saved like any other edit.

## Rehearsal
- The masks take turns: the run asks mask `reviews % masks` (the card's earlier reviews). A card without
  masks rehearses as a plain card.
- Prompt: the front's text without the picture (or "What is hidden under the marked box?") and the
  picture. **Hide all, guess one:** every mask is opaque and the asked one is filled with the accent
  colour and outlined. **Hide one, guess one:** only the asked mask is opaque and marked.
- After the flip or a typed answer (not in an exam, which never shows answers) the asked mask is
  uncovered and outlined, the others stay as before; the answer shows the mask's label in bold, then
  the card's back. A typed answer is compared with the label of the mask that was shown.
