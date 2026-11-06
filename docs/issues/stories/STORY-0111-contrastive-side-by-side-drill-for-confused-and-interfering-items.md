---
id: STORY-0111
type: story
title: Contrastive side-by-side drill for confused and interfering items
status: backlog
epic: EPIC-035
parents: [EPIC-035]
estimate: 3
labels: [m12, rehearsal, report, service, ui]
acceptance_criteria:
  - "Users can launch a Contrastive Drill from a detected interference pair in Reports or by selecting 2-4 related items in the Library"
  - "The contrastive drill view displays the prompts and answers of the selected items side-by-side and prompts the learner to match each prompt to its answer and articulate the key distinction"
  - "Completing a contrastive drill optionally creates or updates a linked comparison note or disambiguation hint on the participating items"
---

## Context
Cognitive psychology research on **interleaving** and **discriminative contrast** (Brunmair & Richter, 2019; Carvalho & Goldstone, 2014) shows that when a learner repeatedly confuses two similar concepts (retroactive/proactive interference—e.g., *Mediator vs. Moderator*, *Sensitivität vs. Spezifität*, *Alpha- vs. Beta-Fehler*), reviewing them in isolation fails to resolve the overlap. Viewing the confused items side-by-side and explicitly contrasting their distinguishing criteria resolves interference rapidly.

## Description
Implement a Contrastive Side-by-Side Drill workflow in StudyFrog:
1. **Entry Points:**
   - From the **Leeches & Interference** report (`STORY-0104`): click `"Compare side-by-side"` on any detected `ConfusionPair`.
   - From the **Items** table or **Editor Relations** tab: select 2–4 items and choose `"Contrastive drill"` from bulk/context actions.
2. **Side-by-Side Contrastive Drill View (`crates/studyfrog-ui`):**
   - Step 1 (**Discrimination Match**): Show the prompts of the 2–4 confused items side-by-side and their shuffled answers below; ask the user to pair each prompt with its correct answer or type the answer for each column simultaneously.
   - Step 2 (**Side-by-Side Reveal & Distinction Note**): Display both full cards side-by-side with diff/keyword highlighting and an optional text field: *"What is the key difference that distinguishes A from B?"*
3. **Persistence:**
   - Saving the distinction note either appends a `> Distinction: ...` callout to both items or creates a linked comparison note connected to both items via `[[Wikilinks]]`.

## Acceptance Criteria
- [ ] Users can launch a Contrastive Drill from a detected interference pair in Reports or by selecting 2-4 related items in the Library
- [ ] The contrastive drill view displays the prompts and answers of the selected items side-by-side and prompts the learner to match each prompt to its answer and articulate the key distinction
- [ ] Completing a contrastive drill optionally creates or updates a linked comparison note or disambiguation hint on the participating items

## Design Notes
- Touchpoints:
  - `crates/studyfrog-service/src/`: helper to save a disambiguation callout or linked comparison note.
  - `crates/studyfrog-ui/src/view/`: contrastive drill modal/view.
- Adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-035, STORY-0104.

## Open Questions
None.
