---
id: BUG-0016
type: bug
title: Metacognitive calibration measured accuracy baseline calculation and default item difficulty and priority
status: backlog
epic: EPIC-034
parents: []
estimate: 5
labels: [m12, report, service, ui]
acceptance_criteria:
  - "The rehearsal metacognitive calibration report calculates a true measured accuracy baseline instead of naively treating all non-Again ratings as 100% accurate"
  - "The accuracy baseline calculation factors in item novelty/rehearsal history, view latency relative to content length and expected reading speed, item classification, and rating distribution"
  - "Newly created and imported items default to base difficulty Hard (Difficult) and priority Medium across all creation paths, and existing items in vaults missing these attributes are normalized consistently"
---

## Context
In the rehearsal reports screen under **Metacognitive Calibration** (comparing subjective learning diary ratings against objective review performance), the report consistently displays `Measured accuracy: 100 %` (e.g. 32 reviews across 2 diary entries showing 46 % self-rated performance, 100 % measured accuracy, and 3 % average familiarity, triggering a distorted "-54.2 % underconfident" assessment).

This occurs because `day_review_metrics` in `crates/studyfrog-report/src/calibration.rs` naively calculates measured accuracy as:
```rust
let recalled = reviews.iter().filter(|r| r.rating != Rating::Again).count();
recalled as f32 / reviews.len() as f32
```
If a learner reviews new cards for the first time and rates them `Hard`, `Good`, or `Easy`, every single card is counted as 100 % recalled. An item that has never been rehearsed or viewed before cannot legitimately represent 100 % baseline accuracy.

Furthermore, learners expect consistent baseline metadata when authoring and organizing study material:
1. When any new item is created or imported, its default difficulty should be **Hard** (`Difficulty::Difficult` / FSRS difficulty ~7.0) and its priority should be **Medium** (`Priority::Medium`).
2. In many existing vaults, imported or legacy items lack these baseline defaults, skewing scheduling calculations and calibration metrics.

## Description
Deliver an accurate baseline calculation for metacognitive accuracy and enforce default difficulty and priority:

1. **Realistic Metacognitive Measured Accuracy Baseline (`studyfrog-report::calibration`):**
   - Replace the simplistic `rating != Rating::Again` ratio with a weighted calibration metric:
     - **Item History & Novelty:** Unreviewed or first-time rehearsed items cannot yield 100 % accuracy on a single pass; their effective accuracy must be tempered by repetition count and familiarity.
     - **View Time vs. Content Size:** Compare actual view latency (`latency_ms`) against expected reading speed based on prompt/answer word count. Hesitant or excessively prolonged responses indicate struggling retrieval even if rated non-Again.
     - **Rating Weighting:** Differentiate ratings: `Easy` (1.0), `Good` (0.8), `Hard` (0.5), `Again` (0.0), combined with familiarity and response latency.
   - Ensure the Metacognitive Calibration card and daily trend rows accurately reflect realistic recall grasp.
2. **Standardized Default Difficulty (Hard) & Priority (Medium) (`studyfrog-service`, `studyfrog-core`, `studyfrog-ui`):**
   - Ensure all item creation pathways (Modal Create, CSV/TSV import, note-to-card inline extraction, AI prompt generation, and CLI/MCP creation) default newly created items to:
     - `difficulty: Difficult` (Hard)
     - `priority: Medium`
   - Provide a vault migration / startup check that normalizes existing vault items missing difficulty or priority attributes.

## Acceptance Criteria
- [ ] The rehearsal metacognitive calibration report calculates a true measured accuracy baseline instead of naively treating all non-Again ratings as 100% accurate
- [ ] The accuracy baseline calculation factors in item novelty/rehearsal history, view latency relative to content length and expected reading speed, item classification, and rating distribution
- [ ] Newly created and imported items default to base difficulty Hard (Difficult) and priority Medium across all creation paths, and existing items in vaults missing these attributes are normalized consistently

## Design Notes
Touchpoints across crates:
- `crates/studyfrog-report`: `calibration.rs`, `calibration_types.rs`, `familiarity.rs`.
- `crates/studyfrog-service`: `create.rs`, `xfer_csv_map.rs`, `content.rs`.
- `crates/studyfrog-ui`: `view/reports_calibration.rs`, `modal_form.rs`.
- Guardrails: Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-024 (Diary and metacognitive calibration).
- EPIC-034 (Behavioral learning analytics and FSRS optimizer).

## Open Questions
- None.
