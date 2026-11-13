---
id: STORY-0111
type: story
title: "Contrastive side-by-side drill for confused and interfering items"
status: done
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
- [x] Users can launch a Contrastive Drill from a detected interference pair in Reports or by selecting 2-4 related items in the Library
- [x] The contrastive drill view displays the prompts and answers of the selected items side-by-side and prompts the learner to match each prompt to its answer and articulate the key distinction
- [x] Completing a contrastive drill optionally creates or updates a linked comparison note or disambiguation hint on the participating items

## Design Notes
- Touchpoints:
  - `crates/studyfrog-service/src/`: helper to save a disambiguation callout or linked comparison note.
  - `crates/studyfrog-ui/src/view/`: contrastive drill modal/view.
- Adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and verified:
- What STORY-0104 had: Reports > Leeches lists confused pairs with "Drill A and B together" (a practice run of the
  two items). This story adds the side-by-side drill next to it.
- Service `crates/studyfrog-service/src/contrast.rs` and `contrast_save.rs`: load 2-4 items; keep the distinction as
  a `> Distinction (vs [[...]])` hint on each item (replaced on a later save, ignored when typed answers are
  compared) or as a comparison note with `compares` that is updated rather than duplicated; undo snapshot.
  Fixture tests `crates/studyfrog-service/tests/contrast.rs`.
- Window: `crates/studyfrog-ui/src/contrast.rs` (shuffle without an answer under its own prompt, matching, checking,
  words only each answer has) and `view/contrast.rs`; entry points in Reports > Leeches ("Compare A and B side by
  side") and the Library bulk bar ("Contrastive drill" for 2-4 rows). Harness and real-worker tests
  `crates/studyfrog-ui/tests/recall_contrast_ui.rs`; screenshots `--screen contrast:<ULID>,<ULID>` and
  `--screen compare:<ULID>,<ULID>` looked at.
- Spec `docs/specs/contrastive-drill.md`.

Not built: an entry point on the item page's Relations tab (the Library and Reports cover the criterion); the
"diff" highlighting is word-level ("Only here: ..."), not a character diff.

## Dependencies
EPIC-035, STORY-0104.

## Open Questions
None.
