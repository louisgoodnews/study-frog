# Colour, type, spacing and focus mode (EPIC-036)

Written 2026-11-08 while building EPIC-036 (STORY-0113 to STORY-0116). The research behind it is
`docs/research/ui-aesthetics-and-ux-science.md`; this page says what was built, the numbers, and how each rule is tested.

## Colour (STORY-0113)

### Perceptual colour maths, in-house
`crates/studyfrog-ui/src/color_science.rs` holds pure functions: sRGB to linear light and back, Oklab and OKLCH
(Ottosson 2020, the standard matrices), the largest chroma sRGB can show at a lightness and hue (bisection), OKLCH to
sRGB with gamut mapping (chroma is lowered until the colour fits; lightness and hue stay), mixing two colours in Oklab,
and the APCA lightness contrast `apca_lc(text, background)` (APCA-W3 0.0.98G-4g constants: 2.4 power, 0.56/0.57 and
0.65/0.62 exponents, black soft clamp at 0.022, scale 1.14, offset 0.027, clip below 0.1).

Build or buy: the `palette` and `oklab` crates do the Oklab part and there is no maintained APCA crate; the whole thing is
about 160 lines with no dependency, so it is written here. It is tested against published values
(`tests/color_science.rs`): Oklab of white, black and the three primaries (CSS Color 4 / Ottosson), OKLCH of red, and the
eight example pairs of the APCA reference implementation in both polarities (to 0.05 Lc); property tests check that
every 8-bit colour survives sRGB to OKLCH and back, that more OKLCH lightness never gives a darker colour, that greys
have no chroma, and that swapping text and background flips the sign of APCA.

### The palettes are OKLCH tokens
`crates/studyfrog-ui/src/theme_tokens.rs` writes each of the four base palettes (dark, dark high contrast, light, light
high contrast) as OKLCH values; `theme.rs` computes all 24 palettes (2 themes x 2 contrasts x 6 accents) once and hands
them out from a table, so drawing costs nothing extra.

| Token | Dark | Dark, high contrast | Light | Light, high contrast |
|---|---|---|---|---|
| background (wells: text fields, code) | L 0.205 | L 0.10 | L 0.950 | L 0.94 |
| panel (the page, bars) | L 0.235 | L 0.15 | L 0.975 | L 0.97 |
| surface (cards, popups) | L 0.265 | L 0.20 | L 1.000 | L 1.00 |
| surface_hover (quiet fills, hover) | L 0.295 | L 0.30 | L 0.925 | L 0.92 |
| border | L 0.47 | L 0.74 | L 0.86 | L 0.40 |
| text | L 0.93 | L 1.00 | L 0.24 | L 0.00 |
| text_weak | L 0.78 | L 0.90 | L 0.52 | L 0.33 |
| on_accent | L 0.14 | L 0.00 | L 1.00 | L 1.00 |
| accents and status colours | L 0.77, C 0.13 | L 0.85, C 0.12 | L 0.48, C 0.15 | L 0.42, C 0.16 |

Greys have a slight cool hue (265°, chroma at most 0.014). The hues: blue 258°, green 150°, purple 300°, orange 55°,
pink 355°, teal 195°; success 150°, warning 75°, danger 27°, alt 300°. All six accents and the four status colours of a
theme share one lightness; they share one chroma where sRGB can show it (teal and blue at a light theme's lightness
cannot, so they get the most sRGB allows). Surfaces climb background, panel, surface in equal lightness steps
(0.03 dark, 0.025 light, 0.05 dark high contrast, 0.03 light high contrast); the hover fill is at least one step away
from the surface.

The six text colours of a note (`theme_hue.rs`) are OKLCH too: L 0.80 C 0.13 on a dark theme, L 0.45 C 0.14 on a light
one (grey with 15 % of that chroma); high contrast shows them in the text colour. An item's colour tints a surface by
mixing in Oklab (`item_look.rs`), at most 14 % (dark) or 10 % (light), and less if the text would fall below WCAG 4.5:1
or below the APCA targets for body and secondary text.

### APCA targets (tested in `tests/appearance_apca.rs`, for all 24 palettes)
| Pair | Standard | High contrast |
|---|---|---|
| text on background, panel, surface, surface_hover | \|Lc\| >= 75 | \|Lc\| >= 90 |
| text on background in the dark theme (no halation) | \|Lc\| <= 100 | — |
| text_weak on background, panel, surface | \|Lc\| >= 58, and below the text's | \|Lc\| >= 75 |
| on_accent on accent (button labels) | \|Lc\| >= 60 | \|Lc\| >= 75 |
| accent, success, warning, danger, alt as text on background, panel, surface | \|Lc\| >= 45 | \|Lc\| >= 60 |
| border on panel and surface (dividers) | 15 <= \|Lc\| <= 28 | \|Lc\| >= 50 |
| note text colours on panel and surface | \|Lc\| >= 60 | \|Lc\| >= 90 |
| text / text_weak on an item-tinted card | >= 75 / >= 58 (or the plain surface's, if less) | >= 90 |

The WCAG 2 ratios of `tests/appearance_palette.rs` (4.5:1 text, 7:1 high contrast, 3:1 accents) still hold as well.
Measured, standard dark: text -90, weak text -60 to -62, button label on the accent 61 to 66, borders -15 to -17;
standard light: text 98 to 103, weak text 67 to 78, labels on the accent -85 to -89, borders 20 and 25.
