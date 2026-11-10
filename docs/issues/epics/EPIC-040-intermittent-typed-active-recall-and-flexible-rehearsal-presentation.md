---
id: EPIC-040
type: epic
title: Intermittent typed active recall and flexible rehearsal presentation
status: done
parents: []
labels: [m12, rehearsal, core, ui]
acceptance_criteria:
  - "The rehearsal setup screen allows learners to configure typed written recall with variable sampling frequency or randomized presentation per session"
  - "The rehearsal engine and session state support intermittent typed recall alongside standard flip-card reveal and reading notes in the same run"
  - "Scoring, latency, and pacing telemetry accurately reflect whether an item prompt was presented as typed recall or flipped card"
---

## Context
StudyFrog supports distinct rehearsal modes: `Flip cards` (mental recall followed by card reveal and self-rating), `Type answers` (mandatory typed text submission scored against reference answers by the SLM), and `Exam` (strict one-way test with percentage grading).

In long study sessions or large decks, requiring typed answers for every single item causes typing fatigue and slows down review velocity. Conversely, pure flip-card mode can sometimes lead to passive familiarity illusions where learners believe they know an answer without truly retrieving it.

Cognitive science and retrieval-practice literature show that intermittent, unpredictable testing (variable-ratio retrieval practice) keeps learners cognitively alert and maximizes retention while keeping session fatigue low. Learners want to enable typed answers randomly across their selection so that some cards and questions demand written recall while others allow quick flip review.

## Description
Deliver intermittent typed active recall and presentation flexibility during rehearsal runs:

1. **Rehearsal Setup & Run Plan (`studyfrog-core::plan`, `studyfrog-ui::view::setup_how`):**
   - When configuring rehearsal mode, allow selecting typed recall with a sampling frequency (e.g. "Every item (100%)", "Random 50%", "Random 25%", "Random 75%", or a custom percentage).
   - Ensure notes are recognized as reading items and never forced into text comparison.
2. **Deterministic Seeded Selection (`studyfrog-rehearsal`):**
   - When constructing the session queue, use the run's random seed to deterministically assign whether each card and question requires typed recall or flip review.
   - Maintain reproducible session behavior when replaying or resuming.
3. **Dynamic Rehearsal UI (`studyfrog-ui::view::run_card`):**
   - Adapt the active card view dynamically per item: display text input and "Submit answer" when typed recall is sampled, or standard "Show answer" flip controls when not.
   - Show a subtle visual badge or icon indicating typed vs flip expectations.
4. **Telemetry & Scoring Consistency (`studyfrog-rehearsal::session`, `studyfrog-slm`):**
   - Appropriately record SLM score and semantic comparison for typed items, while recording standard self-ratings for flipped items.

## Acceptance Criteria
- [x] The rehearsal setup screen allows learners to configure typed written recall with variable sampling frequency or randomized presentation per session
- [x] The rehearsal engine and session state support intermittent typed recall alongside standard flip-card reveal and reading notes in the same run
- [x] Scoring, latency, and pacing telemetry accurately reflect whether an item prompt was presented as typed recall or flipped card

## Design Notes
Child stories:
- STORY-0136: Make typed answers random per selection during rehearsal (estimate: 3)

Touchpoints across crates:
- `crates/studyfrog-core`: `RunPlan` and `RunMode` options (e.g. `typed_ratio: Option<f32>` or sampling mode).
- `crates/studyfrog-rehearsal`: `Session` item presentation mode resolver based on run seed and item kind.
- `crates/studyfrog-ui`: `view/setup_how.rs` sampling selector, `view/run_card.rs` adaptive controls.

Architecture rules:
- Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-007 (Rehearsal engine).
- EPIC-013 (Comparison scoring).
- EPIC-033 (Exam run mode).

## Open Questions
- None.
