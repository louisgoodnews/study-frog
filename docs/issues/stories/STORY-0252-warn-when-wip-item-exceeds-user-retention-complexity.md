---
id: STORY-0252
type: story
title: "Warn via toast or modal when a WIP new item exceeds the complexity the user usually retains"
status: done
epic: EPIC-077
parents: ['EPIC-077']
estimate: 5
labels: [m14, analytics, service, ui]
acceptance_criteria:
  - 'A pure domain/service estimator computes the user retained-complexity baseline (word count, list/clause density, and text_complexity of well-retained items vs leeches) from the active vault'
  - 'When creating a new Card or Question, StudyFrog compares the WIP draft complexity against the user baseline in real time without blocking input'
  - 'When the WIP draft becomes significantly more complex than the user typically retains, a dismissible toast or modal banner notifies the user with a concise, data-backed explanation and option to continue or simplify/split'
  - 'The warning triggers at most once per draft (until reset/dismissed) and stays silent when the vault lacks sufficient review history'
  - 'Unit and headless UI tests verify baseline estimation, threshold triggering, dismissal, and accessible widget names'
---

## Context
Users often author overly long or multi-concept flashcards and questions that exceed their personal retention sweet spot, leading to repeated lapses later. StudyFrog already has the user's review history, familiarity scores, and item complexity metrics locally.

## Description
Implement real-time WIP complexity warnings in the item creation flow (`modal_create.rs` / `modal.rs`):
1. **Baseline Computation (`studyfrog-report` / `studyfrog-service`)**:
   - Compute a lightweight `RetentionComplexityProfile` (median & upper-quartile complexity of well-retained items, minimum sample threshold, and average complexity of high-lapse items) when opening the Create dialog or loading vault stats.
2. **WIP Draft Scorer (`studyfrog-report` or `studyfrog-core`)**:
   - Pure function scoring WIP `front`/`back` (or `prompt`/`reference`) on word count, bullet/list count, and sentence/clause density, returning a clear `ComplexityWarning` when the draft exceeds the user's retained threshold.
3. **Create Dialog Toast / Banner (`studyfrog-ui`)**:
   - Surface a calm, non-blocking toast or modal notice when the threshold is crossed, explaining the comparison against the user's own retained average, with a one-click **Dismiss** (and option to ignore for the current draft).

## Acceptance Criteria
- [x] A pure domain/service estimator computes the user retained-complexity baseline (word count, list/clause density, and `text_complexity` of well-retained items vs leeches) from the active vault
- [x] When creating a new Card or Question, StudyFrog compares the WIP draft complexity against the user baseline in real time without blocking input
- [x] When the WIP draft becomes significantly more complex than the user typically retains, a dismissible toast or modal banner notifies the user with a concise, data-backed explanation and option to continue or simplify/split
- [x] The warning triggers at most once per draft (until reset/dismissed) and stays silent when the vault lacks sufficient review history
- [x] Unit and headless UI tests verify baseline estimation, threshold triggering, dismissal, and accessible widget names

## Design Notes
- Reuse `studyfrog-store::telemetry` / `library_rows` / `analytics_leech` signals without blocking the UI thread (compute baseline asynchronously on worker and cache in `State`).
- Respect `palette_of(ui)` warning tokens, Invariant P1 (zero inherent `impl` blocks), and Invariant P6 ($\le 250$ LOC per file).

## Dependencies
- `EPIC-077`

## Open Questions
- Whether to default to an inline warning banner inside the Create dialog (which never gets covered by the modal backdrop) or a floating toast, plus a settings toggle to mute complexity nudges.
