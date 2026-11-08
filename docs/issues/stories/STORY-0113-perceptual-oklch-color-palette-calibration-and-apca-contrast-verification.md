---
id: STORY-0113
type: story
title: Perceptual OKLCH color palette calibration and WCAG 3 APCA contrast verification
status: done
epic: EPIC-036
parents: [EPIC-036]
estimate: 5
labels: [m12, core, ui]
acceptance_criteria:
  - "All six accent colors, semantic status colors, and surface elevation steps in studyfrog-ui::theme are calibrated using Oklab/OKLCH perceptual lightness (L) and chroma (C)"
  - "A pure APCA perceptual contrast function (apca_lc) verifies that body text achieves |Lc| >= 75, secondary text achieves |Lc| >= 58, and subtle dividers stay within |Lc| 15..=28 in standard themes"
  - "Automated unit tests enforce perceptual contrast bounds across Dark, Light, Dark High-Contrast, and Light High-Contrast modes for every accent"
---

## Context
As documented in `docs/research/ui-aesthetics-and-ux-science.md`, sRGB channel numbers do not correspond to human brightness perception: sRGB blue appears dramatically darker than sRGB green or orange at equal RGB values, causing uneven badge legibility and inconsistent hover states when switching accents. Furthermore, WCAG 2.x contrast ratios ignore polarity (dark-on-light vs. light-on-dark halation). Calibrating `theme.rs` in **OKLCH** and verifying contrast with **WCAG 3 APCA ($L_c$)** guarantees uniform visual harmony and effortless glyph decoding across all themes.

## Description
Calibrate StudyFrog's color system in OKLCH and verify perceptual contrast with APCA:
1. **Pure OKLCH & APCA Utilities (`crates/studyfrog-ui/src/color_science.rs`):**
   - Implement pure `oklch_to_rgb(l: f32, c: f32, h_deg: f32) -> Color32` and `rgb_to_oklch(c: Color32) -> (f32, f32, f32)` conversions (using standard Oklab matrices, zero external crate dependencies).
   - Implement pure `apca_lc(text: Color32, bg: Color32) -> f32` computing the WCAG 3 APCA perceptual lightness contrast $L_c \in [-108.0, 106.0]$.
2. **Iso-Luminant Palette & Surface Elevation (`crates/studyfrog-ui/src/theme.rs`):**
   - Tune the 4 base palettes (`dark`, `dark + high_contrast`, `light`, `light + high_contrast`) and 6 accents (`Blue`, `Green`, `Purple`, `Orange`, `Pink`, `Teal`) so all accents share matched perceptual lightness ($L \approx 0.74$ in Dark mode, $L \approx 0.48$ in Light mode) and matched chroma.
   - Ensure stepped surface elevation (`background` -> `panel` -> `surface` -> `surface_hover`) maintains uniform $\Delta L$ steps.
3. **Automated Perceptual Contrast Gate (`crates/studyfrog-ui/tests/`):**
   - Add property/unit tests asserting APCA $|L_c|$ bounds for `text`, `text_weak`, `on_accent`, `success`, `warning`, `danger`, and `border` across all `2 × 2 × 6 = 24` theme/contrast/accent combinations.

## Acceptance Criteria
- [x] All six accent colors, semantic status colors, and surface elevation steps in studyfrog-ui::theme are calibrated using Oklab/OKLCH perceptual lightness (L) and chroma (C)
- [x] A pure APCA perceptual contrast function (apca_lc) verifies that body text achieves |Lc| >= 75, secondary text achieves |Lc| >= 58, and subtle dividers stay within |Lc| 15..=28 in standard themes
- [x] Automated unit tests enforce perceptual contrast bounds across Dark, Light, Dark High-Contrast, and Light High-Contrast modes for every accent

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/color_science.rs`: pure Oklab/OKLCH and APCA $L_c$ functions.
  - `crates/studyfrog-ui/src/theme.rs`: updated palette values and elevation steps.
- Strictly adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-036.

## Open Questions
None.

## Progress (2026-11-08)
Done. `color_science.rs` (Oklab/OKLCH, gamut mapping, Oklab mixing, APCA `apca_lc`, no crate: the `palette`/`oklab`
crates cover only Oklab and no maintained APCA crate exists, so 160 lines in-house won), `theme_tokens.rs` (the four base
palettes as OKLCH tokens, 24 palettes computed once), note colours and item tints in OKLCH/Oklab with an APCA guard.
Verified by `tests/color_science.rs` (reference values of Oklab, OKLCH and the APCA-W3 examples; property tests for round
trips, monotonic lightness, greys and polarity) and `tests/appearance_apca.rs` (every pair of the target table in
`docs/specs/ui-colour-and-type.md` in all 24 palettes, iso-lightness and chroma of accents and status colours, equal
elevation steps, note colours, item tints); the WCAG test `tests/appearance_palette.rs` still passes. Screenshots of
Home, the item page, a run and Settings in dark and light looked at: the dark theme keeps its look (borders slightly
stronger), the light theme now has a light-grey page with white cards and a deeper blue accent.
