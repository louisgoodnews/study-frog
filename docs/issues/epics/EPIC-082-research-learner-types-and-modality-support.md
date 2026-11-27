---
id: EPIC-082
type: epic
title: "Research into learner types (auditory, visual and others) and how StudyFrog could support them"
status: backlog
parents: []
labels: [m14, docs, rehearsal]
acceptance_criteria:
  - 'A research note in docs/research/ reviews the evidence on learner types and modality preference, including the studies that found no benefit from matching instruction to a preferred style, with sources cited'
  - 'The note separates what the evidence supports (multimodal presentation, dual coding, retrieval practice) from what it does not (matched-style gains), and records the sources it used'
  - 'A strategy in docs/specs/ or docs/research/ sets out how StudyFrog could offer modality-based study without promising matched-style gains: which features, what the user chooses, what is measured'
  - 'Follow-up stories for the adopted parts are listed under Decisions, each with its acceptance criteria'
---

## Context
Learner-type labels (auditory, visual, kinesthetic and similar) are widespread, but controlled studies have repeatedly found that matching instruction to a claimed style does not improve learning, while multimodal presentation and retrieval practice do. StudyFrog should base any learner-type feature on that evidence, and should say so to the user.

## Description
- Collect and review the research on learner types, modality preference, multimodal learning and dual coding, with an explicit reading of the matched-style studies.
- Identify which StudyFrog features already use more than one modality (text, cards, diagrams, the PDF viewer, planned voice in EPIC-083) and where a user could pick a presentation.
- Propose how the app could support user-chosen modalities: which views, which rehearsal formats, and what to measure to see whether a choice helps that user.
- Keep the outcome honest: the app never assigns a learner type, and any claim about benefit is tested with the user's own results.

## Acceptance Criteria
- [ ] A research note in docs/research/ reviews the evidence, including the matched-style studies, with sources cited
- [ ] The note separates supported from unsupported claims
- [ ] A strategy sets out the user-chosen modality features, what the user chooses, and what is measured
- [ ] Follow-up stories for the adopted parts are listed under Decisions

## Design Notes
- Research output lives in docs/research/ (as ui-guidelines.md and learning-science.md do); the strategy in docs/specs/ once it is decided.
- No new crates; this epic is research and documents, and its stories are where code changes start.

## Dependencies
- EPIC-083 (voice) and EPIC-084 (PDF) supply two of the modalities.

## Decisions
_None yet._

## Open Questions
- Which modalities are in scope for v1.0.0, and which wait for later phases?
- How should a modality choice be measured without a separate study trial in the app?
