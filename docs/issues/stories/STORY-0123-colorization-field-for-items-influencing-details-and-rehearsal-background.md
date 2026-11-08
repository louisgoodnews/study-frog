---
id: STORY-0123
type: story
title: Colorization field for items influencing details and rehearsal view background
status: done
epic: EPIC-032
parents: [EPIC-032]
estimate: 3
labels: [m12, core, md, ui]
acceptance_criteria:
  - "Items support a colorization field in front-matter and in the Item Details Properties tab"
  - "When an item has a color assigned, the Item Details view and Editor container render a subtle, theme-aware background tint derived from that color"
  - "During a rehearsal run, the Rehearsal card surface is tinted with the active item's colorization while preserving APCA/WCAG text contrast in both dark and light themes"
---

## Context
Learners benefit from strong visual cues when viewing, editing, and rehearsing items across different topics, urgency levels, or conceptual categories. Adding a dedicated colorization field to items that directly tints the background of both the **Item Details / Editor view** and the **Rehearsal card view** gives each colored item an immediate, recognizable visual atmosphere while studying.

## Description
Implement the item colorization field and apply its background styling to the Details and Rehearsal views:
1. **Item `color` Field & Details Properties Selector (`crates/studyfrog-core`, `crates/studyfrog-md`, `crates/studyfrog-ui`):**
   - Persist `color: Option<String>` in item front-matter (supporting curated semantic/pastel color names such as `blue`, `green`, `amber`, `rose`, `purple`, `teal`, `slate` or `#RRGGBB` hex values).
   - In `crates/studyfrog-ui/src/view/item_details/tab_properties.rs`, add a **Color** row with clickable color swatch chips and a `None` reset option.
2. **Details View & Editor Background Tinting (`crates/studyfrog-ui/src/view/editor.rs`, `view/item_details/`):**
   - Blend the item's selected color into the panel/card background surface (`theme::tint` / OKLCH alpha blend at ~10–14% opacity in dark mode and ~8–12% opacity in light mode) so the Details notebook and item viewer surface reflect the item's color without reducing text contrast.
3. **Rehearsal Card View Background Tinting (`crates/studyfrog-ui/src/view/rehearse.rs`):**
   - Look up the current rehearsal item's `color` field and tint the central rehearsal card frame's background fill and border accent with the item's color during both the prompt phase and the revealed answer phase.

## Acceptance Criteria
- [x] Items support a colorization field in front-matter and in the Item Details Properties tab
- [x] When an item has a color assigned, the Item Details view and Editor container render a subtle, theme-aware background tint derived from that color
- [x] During a rehearsal run, the Rehearsal card surface is tinted with the active item's colorization while preserving APCA/WCAG text contrast in both dark and light themes

## Design Notes
- Complements `STORY-0097` under `EPIC-032` by explicitly targeting the Details notebook background and Rehearsal card container background.
- Touchpoints:
  - `crates/studyfrog-core/src/doc.rs` & `crates/studyfrog-md/src/`: `color` field in front-matter.
  - `crates/studyfrog-ui/src/view/item_details/tab_properties.rs`: color swatch selector and tinted container background.
  - `crates/studyfrog-ui/src/view/rehearse.rs`: tinted rehearsal card frame.
- Adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built with STORY-0097: the `color` field in front matter and the Color row (swatches and None, plus picker and code) in Details > Properties; the item page container and every Details card are tinted from the item's colour (theme-aware share, 14 % dark / 10 % light, sRGB mix lowered until text keeps 4.5:1); the rehearsal card's fill and border take the current item's colour in the prompt and the answer phase (not while paused).

Verified: `crates/studyfrog-core/tests/item_keys.rs` (colour values), `crates/studyfrog-service/tests/item_color.rs` (rehearsal content and library rows read the colour), `crates/studyfrog-ui/tests/item_color_look.rs` (for every palette word and a 216-colour grid, in dark and light, every accent, with and without high contrast, the tinted card and page keep the text and secondary text at 4.5:1 or at what the plain surface had; palette colours visibly change fill and border; the form reads, writes, quotes and keeps hand-written words), `crates/studyfrog-ui/tests/item_color_ui.rs` (real worker: picking in the Properties tab, saving to the file, None removes it, the library dot; the rehearsal colour in the prompt and answer phases, none while paused); screenshots of the item page, the Library and a running rehearsal in the dark and the light theme (`studyfrog --screenshot ... --screen item:<ULID>|library|run --theme dark|light`).

Note: the mix is in sRGB with a WCAG 2 contrast check, not OKLCH/APCA.

## Dependencies
EPIC-032, STORY-0097.

## Open Questions
None.
