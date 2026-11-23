---
id: EPIC-036
type: epic
title: "Evidence-based UI aesthetics, perceptual color science, and cognitive-load UX refinement"
status: review
parents: []
labels: [m12, core, ui, docs]
acceptance_criteria:
  - "All theme palettes, accents, semantic badges, and item tints are calibrated in OKLCH perceptual color space and verified against WCAG 3 APCA perceptual contrast (Lc) thresholds in automated tests"
  - "Long-form reading, note editing, and rehearsal views enforce psychophysical typographic line-length bounds (55-75 cpl), a 4-tier modular scale, and tabular numerals for timers and metrics"
  - "Screen layouts minimize extraneous cognitive load and visual edge entropy by replacing nested box borders with tonal surface elevation, Gestalt proximity spacing, and a Zen focus mode"
---

## Context
Empirical human-computer interaction (HCI) and visual psychophysics research (`docs/research/ui-aesthetics-and-ux-science.md`) demonstrates that UI aesthetics directly govern perceived usability (**Aesthetic-Usability Effect**, Tractinsky et al., 2000), pre-attentive visual trust within 17-50 ms (Tuch et al., 2012), and visual search efficiency (Reinecke et al., 2013). In a learning application where working memory ($4 \pm 1$ chunks, Cowan, 2001) must be reserved for **germane load** (active retrieval and conceptual synthesis), any **extraneous UI load**—such as non-uniform sRGB color luminance, excessive 1 px nested borders ("grid cages"), overly wide text lines (>85 characters per line), or competing visual accents—directly impairs study performance.

## Description
Elevate StudyFrog's user interface (`crates/studyfrog-ui`) using empirical psychophysics and UX science across four structured stories:
1. **Perceptual OKLCH Palette Calibration & WCAG 3 APCA Contrast Verification (`STORY-0113`):**
   - Calibrate `Palette` in `crates/studyfrog-ui/src/theme.rs` using Oklab/OKLCH perceptual lightness ($L$) and chroma ($C$) so all 6 accents (`Blue`, `Green`, `Purple`, `Orange`, `Pink`, `Teal`), semantic statuses (`success`, `warning`, `danger`), and item tints share iso-luminant contrast.
   - Implement a pure APCA perceptual contrast calculator ($L_c$) and enforce automated test gates ($|L_c| \ge 75$ for body text, $|L_c| \ge 60$ for secondary text, and $|L_c| \in [15, 25]$ for subtle structural dividers).
2. **Psychophysical Typographic Measure, Modular Scale & Tabular Numerals (`STORY-0114`):**
   - Constrain reading and rehearsal text columns to the empirically optimal **55-75 characters per line** (`620-680 px` at `100%` scale) to prevent return-sweep eye fatigue (Legge & Bigelow, 2011; Dyson, 2004).
   - Standardize a strict 4-tier modular type scale and enforce tabular figures (`monospace` / proportional-tabular metrics) on countdown timers, run clocks, and table numeric columns so digits never jitter horizontally.
3. **Low-Entropy Surface Elevation, Border De-Cluttering & Gestalt Proximity (`STORY-0115`):**
   - Reduce visual edge entropy across Reports, Stack workspace tabs, and the Details notebook by replacing nested 1 px borders with stepped OKLCH surface luminance (`background` -> `panel` -> `surface` -> `surface_hover`) and strict `4 / 8 / 16 / 24 px` Gestalt proximity spacing.
   - Enforce Hick's Law (**Rule of 1 Primary CTA** per view) so only the primary action carries a high-salience accent fill.
4. **Distraction-Free Zen Mode & Zero-Layout-Shift Interaction Polish (`STORY-0116`):**
   - Add a toggleable/automatic **Zen Focus Mode** (`F11` or `Ctrl+Shift+F`) that collapses the left sidebar and secondary chrome during active Rehearsal/Exam runs and full-window Note writing, while guaranteeing Fitts's Law target sizing ($\ge 36\text{ px}$ height for rehearsal rating buttons) and zero layout shift on state transitions.

## Acceptance Criteria
- [x] All theme palettes, accents, semantic badges, and item tints are calibrated in OKLCH perceptual color space and verified against WCAG 3 APCA perceptual contrast (Lc) thresholds in automated tests
- [ ] Long-form reading, note editing, and rehearsal views enforce psychophysical typographic line-length bounds (55-75 cpl), a 4-tier modular scale, and tabular numerals for timers and metrics
- [x] Screen layouts minimize extraneous cognitive load and visual edge entropy by replacing nested box borders with tonal surface elevation, Gestalt proximity spacing, and a Zen focus mode

## Design Notes
Child stories:
- STORY-0113: Perceptual OKLCH color palette calibration and WCAG 3 APCA contrast verification (estimate: 5)
- STORY-0114: Psychophysical typographic measure, modular hierarchy, and tabular numerals (estimate: 3)
- STORY-0115: Low-entropy surface elevation, border de-cluttering, and Gestalt proximity spacing (estimate: 5)
- STORY-0116: Distraction-free Zen focus mode and zero-layout-shift interaction polish (estimate: 3)

Reference research: `docs/research/ui-aesthetics-and-ux-science.md`.

## Dependencies
EPIC-020 (UI Rework) and EPIC-027 (Appearance and UI Settings).

## Open Questions
None.

## Progress (2026-11-08)
In **review** because STORY-0114 is in review (its reading measure is kept at 60 to 75 characters, about 450 pt in
StudyFrog's font, not the 620 to 680 px it names; the owner should accept or ask for the pixel width). STORY-0113,
STORY-0115 and STORY-0116 are done. Criterion 2 is not ticked for the same reason; criteria 1 and 3 are met (see the
stories). Spec `docs/specs/ui-colour-and-type.md`.

## Review (2026-11-22)

Review (2026-11-22): criterion 2 stays unticked. Its line-length bound is stated in characters (55 to 75 cpl); the Help page measured about 60 to 70 cpl (/tmp/claude-0/review-a/help-1280.png), but note preview and rehearsal were not checked here, and the pixel question sits with STORY-0114. Criteria 1 and 3 were not re-checked here. Status stays review.

## Verification (2026-11-23)

Checked in the running app (1280x800, copy at `/tmp/claude-0/g2`) and by the tests at this head.
- Criterion 1 (palettes in OKLCH, APCA Lc in tests): stays ticked. `appearance_apca` (6), `appearance_palette` (1, every palette meets the text and control rules) and `color_science` (11) pass. The Appearance settings show the six accents with legible labels (`/tmp/claude-0/g2/shot-51-appearance.png`). The Lc values were read from the tests, not measured from the pixels.
- Criterion 2 (55 to 75 cpl, 4-tier scale, tabular numerals): NOT ticked. The 4-tier scale, the 1.5 line height and the tabular digits hold (`typography` tests pass; the focus timer pill keeps its digits in place across three frames, `/tmp/claude-0/g2/crop-focus-stack.png`). The measure holds in Help, the note preview and the rehearsal cards (about 60 to 67 characters per line). It does not hold in note editing: the Markdown editor is `desired_width(f32::INFINITY)` (`crates/studyfrog-ui/src/view/md_editor/mod.rs`, line 89), so its lines run the full window width. The criterion names note editing, so it stays open until the editor is bounded or the owner narrows the criterion.
- Criterion 3 (tonal surfaces, Gestalt spacing, Zen): stays ticked. The Zen button in the top bar hides the sidebar and the status bar during a run (`/tmp/claude-0/g2/shot-13-zen-button.png`). Reports (`shot-14-reports.png`) and the stack page (`shot-16-stack.png`) use tonal panels instead of nested 1 px boxes. The Details notebook was not reached in this session (item rows did not open by mouse), so its part is not re-checked here.

Status stays review: criterion 2 is open. STORY-0114 (child) stays review for the same epic.
