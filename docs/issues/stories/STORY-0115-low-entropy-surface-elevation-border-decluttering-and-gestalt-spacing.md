---
id: STORY-0115
type: story
title: "Low-entropy surface elevation, border de-cluttering, and Gestalt proximity spacing"
status: done
epic: EPIC-036
parents: [EPIC-036]
estimate: 5
labels: [m12, ui]
acceptance_criteria:
  - "Nested box-in-box 1 px borders in Reports, Stack Summary, Home, and the Details Notebook are replaced by tonal surface elevation (panel vs. surface) and Gestalt proximity spacing (4 / 8 / 16 / 24 px)"
  - "Every screen enforces Hick's Law and the Signaling Principle with at most one high-salience accent-filled primary action button per view"
  - "Visual complexity audit tests verify that container spacing obeys the Gestalt Rule of Halves (sibling gap < internal card padding < inter-section gap)"
---

## Context
Research by Reinecke et al. (2013) and Tuch et al. (2012) demonstrates that **visual edge entropy**—the density of high-contrast 1 px rectangular borders and competing visual boxes—is the single strongest predictor of perceived visual clutter. By applying **Gestalt grouping laws** (Proximity, Similarity, and Common Region; Wertheimer, 1923; Palmer, 1992) and **Mayer's Signaling Principle**, StudyFrog can group related controls through stepped surface fills and whitespace rather than drawing heavy outlines around every widget.

## Description
Perform a systematic border de-cluttering and spatial rhythm pass across `studyfrog-ui`:
1. **Spatial Rhythm Tokens (`4 / 8 / 12 / 16 / 24 px`):**
   - Define shared spacing constants in `crates/studyfrog-ui/src/theme.rs`:
     - `GAP_TIGHT: f32 = 4.0` (label to value / icon to text)
     - `GAP_ITEM: f32 = 8.0` (sibling controls within a group)
     - `PAD_CARD: f32 = 16.0` (internal card padding)
     - `GAP_SECTION: f32 = 24.0` (separation between distinct visual groups)
   - Scale these tokens proportionally with the user's configured `Density` (`Compact`, `Comfortable`, `Spacious`).
2. **Tonal Elevation Over Nested Borders:**
   - In `view/home.rs`, `view/reports.rs`, `view/stack_view.rs`, and `view/item_details/`: remove redundant inner `1 px` strokes on sub-cards and stat pills, relying on `palette.surface` vs. `palette.panel` tonal contrast.
3. **Signaling Principle (Single Primary CTA per View):**
   - Audit all top-level views so only the primary task button uses `palette.accent` fill; secondary buttons use quiet `palette.surface_hover` fills.

## Acceptance Criteria
- [x] Nested box-in-box 1 px borders in Reports, Stack Summary, Home, and the Details Notebook are replaced by tonal surface elevation (panel vs. surface) and Gestalt proximity spacing (4 / 8 / 16 / 24 px)
- [x] Every screen enforces Hick's Law and the Signaling Principle with at most one high-salience accent-filled primary action button per view
- [x] Visual complexity audit tests verify that container spacing obeys the Gestalt Rule of Halves (sibling gap < internal card padding < inter-section gap)

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/theme.rs`, `view/home.rs`, `view/reports.rs`, `view/stack_view.rs`, `view/item_details/`.
- Enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-036, STORY-0113.

## Open Questions
None.

## Progress (2026-11-08)
Done. Cards are borderless tonal steps (`surface` on `panel`, nested cards an inset of `panel`), controls are drawn by
fill (outlines on hover, focus and with high contrast), spacing tokens 4 / 8 / 16 / 24 scale with density and drive
egui's item spacing, card padding, captions and section headings (`theme_space.rs`, `view/kit_card.rs`); the top bar's
Create, Home's diary "Write entry" and "Add tool server" became quiet accent-label buttons so each view has one
accent-filled action. Verified by `tests/visual_rhythm.rs` (tokens and installed spacing at all densities, the
measured Appearance settings page, outlines per theme and contrast, at most one primary action on 17 views) and by
screenshots of Home, a stack page, Reports, the item page and Settings in dark and light: no card outlines, cards set
off by tone in both themes, one filled button per view. Dialogs and popups were not audited for the one-primary rule.
