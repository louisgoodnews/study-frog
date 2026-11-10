---
id: STORY-0136
type: story
title: Make typed answers random per selection during rehearsal
status: done
epic: EPIC-040
parents: [EPIC-040]
estimate: 3
labels: [m12, rehearsal, ui]
acceptance_criteria:
  - "The Rehearsal Setup screen ('How') provides a setting to randomly sample typed answers across the chosen items instead of requiring typed input for every single item"
  - "Notes in the rehearsal queue are never forced into typed text comparison and are always shown as readable reference notes"
  - "During rehearsal, the card view dynamically alternates between typed answer submission and standard flip/reveal controls based on the randomized selection"
---

## Context
When preparing for rehearsal sessions, learners often want the rigor and active retrieval benefits of typed recall without the cognitive and physical typing fatigue of typing out answers for every single card, question, or note in a large deck.

Currently, selecting `Type answers` (`RunMode::Answer`) enforces text entry on every single card and question, while `Flip cards` (`RunMode::Flip`) never asks for typed input. Providing a randomized sampling option during setup allows learners to experience intermittent active recall testing across their study deck.

## Description
Deliver randomized typed answer selection for rehearsal runs:

1. **Rehearsal Setup Controls (`crates/studyfrog-ui/src/view/setup_how.rs`):**
   - When configuring the presentation mode in Step 2 of run setup, provide an option for randomized typed answers:
     - Under "Type answers" or as a dedicated presentation option, offer a sampling frequency selector (e.g., "All items (100%)", "Random 50%", "Random 25%", "Random 75%").
     - Clear helper text explaining that typed answers will be randomly requested per card/question, while remaining cards will be presented as flip cards and notes as readable reference notes.
2. **Data Model & Sampling Logic (`crates/studyfrog-core`, `crates/studyfrog-rehearsal`):**
   - Extend `RunPlan` with a typed sampling ratio (e.g., `typed_sample: Option<u8>` or `typed_ratio: Option<f32>`).
   - In `studyfrog-rehearsal::session`, determine whether the item at the current cursor requires typed entry:
     - If the item is a `Note`: always presentation/flip mode (no typing required).
     - If the item is a `Card` or `Question`: evaluate deterministic pseudo-random sampling using `(run.plan.seed, item_uid)` so that replay and pause/resume behave consistently.
3. **Adaptive Card View (`crates/studyfrog-ui/src/view/run_card.rs`):**
   - Adapt the reveal controls based on whether the current item is designated for typed input or flip:
     - For designated typed items: display text input, "Submit answer", and SLM comparison verdict.
     - For non-typed items: display prompt with "Show answer (Space)" and standard reveal flow.
     - Ensure keyboard shortcuts (`Space`, `Enter`, `1`..`4`) and rating flows work seamlessly across transitions between typed and flip items.

## Acceptance Criteria
- [x] The Rehearsal Setup screen ('How') provides a setting to randomly sample typed answers across the chosen items instead of requiring typed input for every single item
- [x] Notes in the rehearsal queue are never forced into typed text comparison and are always shown as readable reference notes
- [x] During rehearsal, the card view dynamically alternates between typed answer submission and standard flip/reveal controls based on the randomized selection

## Design Notes
- **Seeded Determinism:** Random sampling must be derived deterministically from the run's random seed (`RunPlan.seed`) to ensure consistent queue behavior across pauses, crashes, or session exports.
- **Pacing & Latency:** Ensure latency calculation accounts for typing vs. flipping when evaluating reading speed suggestions.
- **Architecture Constraints:**
  - Strictly adhere to P1 (no inherent `impl` blocks).
  - Maximum 250 lines per `.rs` file (P2).

## Dependencies
- EPIC-040 (Intermittent typed active recall and flexible rehearsal presentation).
- STORY-0128 (Exam mode in rehearsal screens).

## Open Questions
- None.
