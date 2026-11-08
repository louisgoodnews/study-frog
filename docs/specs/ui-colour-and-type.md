# Colour, type, spacing and focus mode (EPIC-036)

Written 2026-11-08 while building EPIC-036 (STORY-0113 to STORY-0116). The research behind it is
`docs/research/ui-aesthetics-and-ux-science.md`; this page says what was built, the numbers, and how each rule is tested.

## Colour (STORY-0113)

### Perceptual colour maths, in-house
`crates/studyfrog-ui/src/color_science.rs` holds pure functions: sRGB to linear light and back, Oklab and OKLCH
(Ottosson 2020, the standard matrices), the largest chroma sRGB can show at a lightness and hue (it steps down, then bisects: the gamut's slice is not convex near its corners, which a plain bisection got wrong for a pure blue), OKLCH to
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

## Type (STORY-0114)

### Four sizes
`crates/studyfrog-ui/src/typo.rs` defines the scale; egui's text styles are set from it (`theme_apply.rs`) and every
size follows the user's text size (the body size is 15 pt at 100 %):

| Tier | Share of body | At 100 % | Used for | egui style |
|---|---|---|---|---|
| Heading | 1.40 | 21 pt | page titles, section headings, Markdown headings of levels 1 and 2, the prompt of a run, big numbers | `Heading` |
| Body | 1.00 | 15 pt | prose, notes, answers, labels, Markdown headings of level 3 and lower (strong) | `Body` |
| Label | 0.92 | 13.8 pt | buttons, navigation, captions above groups, code | `Button`, `Monospace` |
| Caption | 0.80 | 12 pt | pills and badges, keyboard hints, timestamps, calendar chips | `Small` |

A level-1 Markdown heading has a hairline under it, which tells it apart from level 2 at the same size. No view writes
a size of its own: `tests/typography.rs` scans the sources and fails on a literal size outside the scale. Four glyphs
drawn as pictures keep theirs (the `?` icon, the two resize grips, the digit in the notification dot).

### Line height and measure
Prose (paragraphs, list items, headings, table cells of the Markdown preview, help pages, item pages, run cards) has a
line height of 1.5 times its size, and the first line of a paragraph is exactly as tall as the others (the wrapping
row height is set before the row starts). Prose is never wider than the **reading measure**: 70 average characters of
body text, measured at run time on a sample of English prose in the font in use (`typo::measure`), so it follows the
text size and any font change. With egui's built-in font at 15 pt this is about 450 pt; full lines average 60 to 75
characters at every offered text size (tested, single lines move a few characters with word wrap). Help pages are a
centred column of that width; a rehearsal card is that wide plus its padding; the item page's preview keeps
paragraphs to it inside a wider pane.

The research figure of 620 to 680 px assumes a wider font at about 16 px; in StudyFrog's font that width would hold
about 100 characters, so the measure is kept in characters, not pixels.

### Numbers that do not jitter
egui cannot switch on OpenType features such as `tnum`, but both built-in fonts already have tabular digits: every
digit has the same advance width (tested for the proportional and the monospace font). So a number changes width
only when it gains a character. The ticking places therefore reserve the width of their widest form: the seconds on
the current item ("000.0 s"), the run's position ("10 / 10" for a run of ten), the run clock, and the focus timer in
the top bar (`kit::figure_pill`, `typo::figure_template`). Numeric columns of the Library table (reviews,
familiarity) are right-aligned so their digits line up.

## Surfaces and spacing (STORY-0115)

### Depth by tone, not outlines
A card (`kit::card`, `kit::group`, `kit::card_button`, in `view/kit_card.rs`) is the `surface` step on the `panel`
page and has **no border**; a card inside a card is an inset of the `panel` step, also without a border. Buttons,
combo boxes and selectable labels are drawn by their quiet `surface_hover` fill alone (their outline is the fill colour,
so a text field keeps a faint ring around its darker well); hover draws an accent outline and focus the selection
outline, neither of which changes a control's size. Separators keep the `border` hairline. **High contrast** brings
every outline back in the strong border colour. A card tinted by an item's colour keeps its colour border, because the
colour means something. `tests/visual_rhythm.rs` checks the outlines of both themes with and without high contrast.

### Rhythm
Four tokens (`theme.rs`, scaled by density in `theme_space.rs`: compact 0.75, comfortable 1, spacious 1.25):

| Token | Comfortable | Between |
|---|---|---|
| `GAP_TIGHT` | 4 | a caption and its card, a label and its value |
| `GAP_ITEM` | 8 | sibling controls (egui's item spacing is set to it) |
| `PAD_CARD` | 16 | a card's edge and its content |
| `GAP_SECTION` | 24 | one group and the next (`kit::caption` and `kit::section` bring it) |

Each is twice the one before, so the rule of halves (siblings closer than a card's padding, padding closer than the
next group) holds at every density. Tested on the tokens, on egui's installed spacing, and measured on the drawn
Appearance settings page (sibling gap 8, card padding 16, group gap 24).

### One primary action
Only `kit::primary_button` fills a button with the accent, and it counts itself each frame (`theme_space::note_primary`).
`tests/visual_rhythm.rs` draws Home, Folders, Library, Rehearse, Settings, Help, Calendar, Board, Reports, Diary,
Assistant, Prompts, Achievements, Graph, a stack page, a flip run and a typed run, and fails if any shows more than one.
Secondary actions that should still be found (the top bar's **+ Create**, Home's diary **Write entry**, **Add tool
server**) are `kit::quiet_button`: the quiet fill with an accent label. Dialogs and popups have their own primary.

## Zen mode and steady targets (STORY-0116)

- **Zen mode** (`zen.rs`, `view/zen.rs`) leaves out the sidebar and the status bar; the page takes their room and the
  run card stays centred. It is toggled by **Ctrl+Shift+F** or **F11** on any screen (also while a text field has the
  keyboard) and by the four-corners button in the top bar, which is highlighted while it is on and is the visible way
  out. The top bar stays: it is StudyFrog's own title bar with the window buttons. The setting `zen_in_runs`
  (Settings > Appearance, "Zen mode during rehearsal and exam runs", `app.toml` `[appearance] zen_in_runs = true`)
  turns it on by itself while a run is on screen; a choice made with the shortcut or button wins until the next run
  starts. Esc keeps its meaning in runs (stop), so it does not leave Zen mode.
- **Targets:** `kit::primary_button` (Show answer, Submit, Resume and every other primary action) is at least
  `MIN_TARGET` = 36 pt tall (or the density's control height if larger); the rating buttons are 46 pt. Their labels
  carry their keys: "Again (1)" to "Easy (4)", "Show answer (Space)".
- **No layout shift:** the run card is anchored at the top and the answer grows below the rule, so the prompt and
  the header ("1 / 10", mode, timer) keep their places when the answer shows (tested to the point). Outlines on hover
  and focus are drawn without changing a control's size (egui keeps the stroke inside the margin), and the timers
  reserve their widest width (STORY-0114).

Tested in `tests/zen_mode.rs` (shortcuts, button, automatic mode, the sidebar and status bar gone and back, target
heights, header and prompt positions before and after Show answer). Screenshots: a run with `zen_in_runs` on, dark and
light, showed no sidebar and no status bar, the card centred in the full width, and the Zen button highlighted.
