---
id: STORY-0097
type: story
title: "Item colorization with color picker and viewer background styling"
status: done
epic: EPIC-032
parents: [EPIC-032]
estimate: 3
labels: [m12, ui]
acceptance_criteria:
  - "Item frontmatter supports an optional color attribute storing hex color codes or palette tokens"
  - "The editor and details notebook view provide an intuitive color picker with curated palette presets and custom color selection"
  - "Setting an item color renders a subtle, accessible background tint in the rehearsal card viewer, editor, and item list"
---

## Context
Learners frequently categorize study materials into thematic, difficulty, or contextual domains (e.g., green for foundational formulas, blue for historical dates, amber for exam focus areas). Currently, StudyFrog offers tags, but lacks visual colorization. Introducing an item color property with a dedicated color picker allows users to personalize cards and visually distinguish them during study sessions.

## Description
Implement item colorization in the domain model and user interface:
1. Data Model & Frontmatter:
   - Support `color: "#4A90E2"` (or token names) in note/card/question markdown frontmatter.
   - Parse and serialize the color property cleanly in `studyfrog-md` and `studyfrog-store`.
2. Color Picker UI:
   - In `crates/studyfrog-ui/src/view/editor.rs` and `item_details.rs`, provide a color picker widget:
     - Curated palette chips: Soft/pastel colors tuned for both dark and light themes (e.g. Sage, Sky Blue, Rose, Lavender, Amber, Slate, None).
     - Custom hex input or color picker slider for fine-grained selection.
     - "Clear color" button to revert to default background.
3. Viewer & Rehearsal Styling:
   - In the rehearsal card viewer (`crates/studyfrog-ui/src/view/run.rs`), render the card surface with a subtle background tint derived from the item's color (ensuring contrast ratios and WCAG readability standards remain intact).
   - In the browser items table and folders view, display a color dot or pill next to the item title.

## Acceptance Criteria
- [x] Item frontmatter supports an optional color attribute storing hex color codes or palette tokens
- [x] The editor and details notebook view provide an intuitive color picker with curated palette presets and custom color selection
- [x] Setting an item color renders a subtle, accessible background tint in the rehearsal card viewer, editor, and item list

## Design Notes
- Theme accessibility: Color tints must be blended with the background (e.g. 10–15% opacity or alpha blending) so text remains fully legible in both light and dark themes without violating contrast rules.
- Crate touchpoints:
  - `crates/studyfrog-core/src/`: Add `color: Option<String>` to item structs.
  - `crates/studyfrog-md/src/`: Parse and serialize `color` in YAML frontmatter.
  - `crates/studyfrog-ui/src/view/editor.rs`, `view/run.rs`, `view/browser.rs`: Color picker widget and card background styling.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Built:
- `color` in front matter as an unknown key (`studyfrog_md::extra`): a palette word (`sage`, `sky`, `rose`, `lavender`, `amber`, `teal`, `slate`; `green`, `blue`, `purple`, `red`, `gray` are read as aliases) or `"#RRGGBB"` (quoted, since `#` starts a YAML comment). Indexed in `item.color` (migration 24); `core::item_color`.
- The item page (editor and Details notebook) has a Color row in the Properties tab: None, seven palette swatches, a colour picker and a `#RRGGBB` field. A hand-written value is not rewritten unless it is changed.
- Tints (`ui::item_look`): the rehearsal card (fill and border), the item page and its Details cards, and the Library rows with a colour dot before the title; the share is 14 % (dark) / 10 % (light), lowered automatically when the text would fall under 4.5:1.

Verified: `crates/studyfrog-core/tests/item_keys.rs` (colour values), `crates/studyfrog-service/tests/item_color.rs` (rehearsal content and library rows read the colour), `crates/studyfrog-ui/tests/item_color_look.rs` (for every palette word and a 216-colour grid, in dark and light, every accent, with and without high contrast, the tinted card and page keep the text and secondary text at 4.5:1 or at what the plain surface had; palette colours visibly change fill and border; the form reads, writes, quotes and keeps hand-written words), `crates/studyfrog-ui/tests/item_color_ui.rs` (real worker: picking in the Properties tab, saving to the file, None removes it, the library dot; the rehearsal colour in the prompt and answer phases, none while paused); screenshots of the item page, the Library and a running rehearsal in the dark and the light theme (`studyfrog --screenshot ... --screen item:<ULID>|library|run --theme dark|light`).

Not built (description only, not acceptance criteria): a colour dot in the Folders view, a bulk "set colour" in the Library bulk bar.

## Dependencies
EPIC-032.

## Open Questions
None.
