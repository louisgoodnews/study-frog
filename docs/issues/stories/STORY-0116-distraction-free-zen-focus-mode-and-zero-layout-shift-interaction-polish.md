---
id: STORY-0116
type: story
title: Distraction-free Zen focus mode and zero-layout-shift interaction polish
status: done
epic: EPIC-036
parents: [EPIC-036]
estimate: 3
labels: [m12, ui]
acceptance_criteria:
  - "Users can toggle a Zen Focus Mode (via keyboard shortcut or top-bar icon, and optionally auto-enabled during Rehearsal/Exam runs) that hides the left sidebar and status bar chrome"
  - "Primary rehearsal rating buttons (Again, Hard, Good, Easy) and Flip/Submit controls meet Fitts's Law ergonomic target heights (>= 36 px) with clear keyboard badge hints"
  - "Transitioning between prompt and answer states in Rehearsal or opening inline confirmations produces zero vertical layout shift of the primary card header"
---

## Context
Cognitive Load Theory (Sweller, 1988) and selective-attention studies show that peripheral navigation bars, unread badge counters, and status lines compete for visual attention during effortful memory retrieval and long-form writing. Furthermore, Fitts's Law (1954) and the Doherty Threshold (<400 ms) show that generous hit targets and stable, zero-shift card geometry keep learners in an uninterrupted flow state.

## Description
Implement Zen Focus Mode and interaction stability polish in `studyfrog-ui`:
1. **Zen Focus Mode (`State.zen_mode: bool`):**
   - Toggleable at any time via `Ctrl+Shift+F` (or a subtle expand/focus button in the top bar), and optionally enabled automatically when entering an active Rehearsal or Exam run (`settings.auto_zen_in_rehearsal`).
   - When active, smoothly hides the left navigation sidebar and bottom status line, centering the Rehearsal card or Markdown Note editor in a calm, distraction-free canvas while keeping `Esc` / top-right exit controls accessible.
2. **Fitts's Law Target Sizing & Zero-Shift Card Geometry:**
   - Ensure the four rehearsal rating buttons (`Again (1)`, `Hard (2)`, `Good (3)`, `Easy (4)`) and `Show answer (Space)` / `Submit (Enter)` have a minimum height of `36 px` (scaled by UI scale).
   - Anchor the rehearsal card top edge at a fixed vertical offset so revealing the answer expands downward without jumping the question prompt upward.

## Acceptance Criteria
- [x] Users can toggle a Zen Focus Mode (via keyboard shortcut or top-bar icon, and optionally auto-enabled during Rehearsal/Exam runs) that hides the left sidebar and status bar chrome
- [x] Primary rehearsal rating buttons (Again, Hard, Good, Easy) and Flip/Submit controls meet Fitts's Law ergonomic target heights (>= 36 px) with clear keyboard badge hints
- [x] Transitioning between prompt and answer states in Rehearsal or opening inline confirmations produces zero vertical layout shift of the primary card header

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/state.rs`, `view/mod.rs`, `view/topbar.rs`, `view/rehearse.rs`.
- Adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-036.

## Open Questions
None.

## Progress (2026-11-08)
Done. Zen mode (`zen.rs`, `view/zen.rs`): Ctrl+Shift+F, F11 and a top-bar button hide the sidebar and the status bar;
the `zen_in_runs` appearance setting (core, `app.toml`, Settings > Appearance) turns it on during rehearsal and exam
runs. Primary buttons (Show answer, Submit) are at least 36 pt, ratings 46 pt, labels carry their keys. Verified by
`tests/zen_mode.rs` (toggle by both shortcuts and the button, sidebar and status bar gone and back, automatic mode and
leaving it, button heights, prompt and header positions unchanged by Show answer), the config round-trip test, and
screenshots of a run in Zen mode (dark, light). Esc is not used to leave Zen mode because it stops a run; the top bar
stays because it is the window's title bar. Inline confirmations other than the run's answer were not measured.
