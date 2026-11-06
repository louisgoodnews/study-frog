---
id: STORY-0114
type: story
title: Psychophysical typographic measure, modular hierarchy, and tabular numerals
status: backlog
epic: EPIC-036
parents: [EPIC-036]
estimate: 3
labels: [m12, ui]
acceptance_criteria:
  - "Long-form Markdown reading, note preview, help/wiki pages, and rehearsal cards constrain line width to an ergonomic reading measure (620-680 px scaled by text size, ~60-75 characters per line)"
  - "Typography across studyfrog-ui follows a strict 4-tier modular scale (Heading, Body, UI Label, Caption) with comfortable 1.45-1.55x line spacing"
  - "Timers, progress counters, and numeric table columns render with tabular/fixed-width digit alignment so ticking clocks and metrics never jitter horizontally"
---

## Context
Psychophysical reading studies (Legge & Bigelow, 2011; Dyson, 2004) show that lines wider than 80–85 characters significantly increase **return-sweep errors** (eyes losing their place when sweeping to the next line), while lines shorter than 40 characters disrupt foveal/parafoveal phrase chunking. In addition, proportional digits in ticking rehearsal timers (`21.9 s` -> `22.0 s`) cause subtle horizontal layout jitter that draws peripheral attention away from the flashcard prompt.

## Description
Standardize typographic measure, modular scale, and numeric alignment across `studyfrog-ui`:
1. **Ergonomic Reading Measure (`60–75 cpl`):**
   - Define `MAX_READING_WIDTH: f32 = 660.0` (scaled by the user's configured `text_scale`) in `crates/studyfrog-ui/src/theme.rs` or layout helpers.
   - Apply this reading measure to Markdown note previews (`view/preview.rs`), Help & Wiki articles (`view/help.rs`), and Rehearsal card containers (`view/rehearse.rs`) so wide monitors frame content with calm lateral breathing room rather than stretching paragraphs across 1,600+ pixels.
2. **4-Tier Modular Type Scale:**
   - Standardize `TextStyle` sizes in `theme_apply.rs` into four clear semantic tiers:
     1. `Heading`: `1.40×` base (page titles and active rehearsal prompts).
     2. `Body`: `1.00×` base (editor, note prose, card answers).
     3. `Button / Label`: `0.92×` base (navigation items, buttons, table cells).
     4. `Small / Caption`: `0.80×` base (badges, keyboard shortcut hints, timestamps).
3. **Zero-Jitter Tabular Numerals:**
   - Render ticking rehearsal timers, Pomodoro clocks, and numeric badge counters in a fixed-width/tabular style or fixed-width pill container so digit transitions never shift neighboring controls.

## Acceptance Criteria
- [ ] Long-form Markdown reading, note preview, help/wiki pages, and rehearsal cards constrain line width to an ergonomic reading measure (620-680 px scaled by text size, ~60-75 characters per line)
- [ ] Typography across studyfrog-ui follows a strict 4-tier modular scale (Heading, Body, UI Label, Caption) with comfortable 1.45-1.55x line spacing
- [ ] Timers, progress counters, and numeric table columns render with tabular/fixed-width digit alignment so ticking clocks and metrics never jitter horizontally

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/theme_apply.rs`, `view/preview.rs`, `view/rehearse.rs`, `view/help.rs`.
- Keep all `.rs` files ≤ 250 lines (P2) with zero inherent `impl` blocks (P1).

## Dependencies
EPIC-036.

## Open Questions
None.
