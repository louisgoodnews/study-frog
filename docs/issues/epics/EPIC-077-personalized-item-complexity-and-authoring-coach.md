---
id: EPIC-077
type: epic
title: "Personalized item-complexity guardrails during authoring based on user retention history"
status: done
parents: []
labels: [m14, analytics, service, ui]
acceptance_criteria:
  - 'StudyFrog computes a per-user retained-complexity baseline from existing vault history (item length/word count, concept/list density, familiarity, lapse/Again rate, and telemetry text_complexity)'
  - 'While authoring a new item in the Create dialog (or Editor), StudyFrog evaluates the WIP item complexity in real time without blocking typing'
  - 'When a WIP item significantly exceeds the complexity level the user typically retains well, a calm, dismissible toast or inline warning explains why and suggests splitting it into smaller cards/questions'
  - 'Users with insufficient review history fall back gracefully (either silent until enough data exists or a gentle general threshold) and can dismiss or disable complexity nudges'
---

## Context
StudyFrog already records rich per-item retention and complexity signals in the local vault index (`TelemetryPoint.text_complexity`, review ratings/lapses, FSRS difficulty/stability, familiarity, and card/question body length). However, when a user creates a new card or question that is far denser or longer than the items they historically retain, they only discover it weeks later when it becomes a leech.

## Description
Use the user's own local retention history to provide real-time, personalized authoring feedback:
1. **Personal Retained-Complexity Profile (`studyfrog-report`, `studyfrog-service`)**:
   - Correlate item complexity features (word count, sentence/clause count, list/bullet items, cloze count) against retention outcomes (familiarity, lapse/Again rate, empirical difficulty) across reviewed items in the vault.
   - Derive a personalized "comfortable retention ceiling" (e.g. 75th–85th percentile complexity of well-retained items, with a minimum sample floor so new vaults aren't spammed).
2. **Live WIP Complexity Check & Nudge (`studyfrog-ui`)**:
   - Evaluate the WIP `CreateForm` draft (`front` + `back` or `prompt` + `reference`) using a pure, fast complexity scorer as the user writes.
   - Trigger a non-blocking, dismissible toast or calm inline modal/banner once the WIP draft crosses significantly above the user's retained-complexity ceiling (e.g. `> 1.5x` their usual retained complexity), showing concrete numbers (e.g. *"This card is ~85 words; items you retain well average ~28 words — consider splitting into 2–3 focused cards"*).

## Acceptance Criteria
- [x] StudyFrog computes a per-user retained-complexity baseline from existing vault history (item length/word count, concept/list density, familiarity, lapse/Again rate, and telemetry `text_complexity`)
- [x] While authoring a new item in the Create dialog (or Editor), StudyFrog evaluates the WIP item complexity in real time without blocking typing
- [x] When a WIP item significantly exceeds the complexity level the user typically retains well, a calm, dismissible toast or inline warning explains why and suggests splitting it into smaller cards/questions
- [x] Users with insufficient review history fall back gracefully (either silent until enough data exists or a gentle general threshold) and can dismiss or disable complexity nudges

## Design Notes
- Strictly local-first (Invariant P8): computed purely from the local SQLite index and Markdown vault.
- Pure domain scoring in free functions (Invariant P1), files $\le 250$ LOC (Invariant P6).
- Child story: `STORY-0252`.

## Dependencies
- Existing cognitive/retention analytics (`studyfrog-report`, `studyfrog-service/src/cognitive_items.rs`, `analytics_leech.rs`)

## Open Questions
- Should the nudge appear as an inline banner inside the Create dialog, a toast, or an optional modal with a one-click "Split into multiple cards" helper?
- What minimum number of reviewed items (e.g. 15–20) should be required before switching from silent/default to personalized thresholds?
