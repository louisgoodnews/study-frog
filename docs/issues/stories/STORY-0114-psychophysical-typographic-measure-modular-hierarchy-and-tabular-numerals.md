---
id: STORY-0114
type: story
title: Psychophysical typographic measure, modular hierarchy, and tabular numerals
status: review
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
- [x] Typography across studyfrog-ui follows a strict 4-tier modular scale (Heading, Body, UI Label, Caption) with comfortable 1.45-1.55x line spacing
- [x] Timers, progress counters, and numeric table columns render with tabular/fixed-width digit alignment so ticking clocks and metrics never jitter horizontally

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/theme_apply.rs`, `view/preview.rs`, `view/rehearse.rs`, `view/help.rs`.
- Keep all `.rs` files ≤ 250 lines (P2) with zero inherent `impl` blocks (P1).

## Dependencies
EPIC-036.

## Open Questions
None.

## Progress (2026-11-08)
Status **review**: everything is built and tested, but the first criterion names a pixel width that contradicts its own
characters-per-line goal in StudyFrog's font, so the owner should decide.

- **Measure (criterion 1, not ticked):** prose in the Markdown preview (item and note pages), help pages and rehearsal
  cards is never wider than `typo::measure`: 70 average characters of body text measured at run time in the font in
  use, so it scales with the text size. Full lines average 60 to 75 characters at every offered text size
  (`tests/typography.rs`); help paragraphs and a long run answer on a 1600 pt window stay within it
  (`tests/reading_measure_ui.rs`); screenshots of Help, the item page and a run in dark and light show lines of about
  55 to 66 characters. In egui's font at 15 pt that is about 450 pt, not the 620 to 680 px the criterion names (that
  width holds about 100 characters here). Accept the character measure, or ask for the pixel one.
- **Scale (ticked):** four tiers (1.40 / 1.00 / 0.92 / 0.80 of the body size) installed as egui's text styles and
  used for every text size in the views; a source scan fails on any literal size (four picture glyphs excepted); prose
  has a line height of 1.5 with an even first line (tested on the galley; seen in screenshots, which before showed a
  taller first line in every paragraph).
- **Numbers (ticked):** both built-in fonts have tabular digits (tested; egui cannot turn on `tnum`, so a font
  without them would need a monospace fallback); the item seconds, run position, run clock and focus timer reserve
  the width of their widest form; Library's numeric columns are right-aligned.
- Also: the run answer is body text in the text colour (APCA body target) instead of the accent at 21 pt.
