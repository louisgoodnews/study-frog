---
id: EPIC-083
type: epic
title: "Research into voice handling: local models for speech comprehension and emission"
status: backlog
parents: []
labels: [m14, docs, slm]
acceptance_criteria:
  - 'A research note in docs/research/ compares local speech-to-text (speech comprehension) and text-to-speech (speech emission) models on accuracy for the target languages, model size, memory, latency on the owner''s machine, licence and offline use'
  - 'The note states the privacy position: audio never leaves the machine unless the user opts in, with the same provider rules as the assistant'
  - 'A build-versus-buy comparison (P10) records which parts are built on an existing runtime crate and which are not, with the reason'
  - 'A strategy sets out where voice fits: spoken answers in rehearsal, reading cards aloud, dictating notes; with the failure behaviour (no model installed, no microphone)'
  - 'Follow-up stories for the adopted parts are listed under Decisions, each with its acceptance criteria and a benchmark plan under docs/contributing/BENCHMARKING.md'
---

## Context
StudyFrog has a local model path (the assistant and the classroom run on Ollama-style local servers) and a first-run wizard that offers optional model downloads. Voice would let a learner answer aloud and hear cards read back, which matters for auditory learners (EPIC-082) and for accessibility. It also has cost: speech models are large and slow on modest hardware, so the research has to decide whether local voice is practical at all.

## Description
- Survey local speech-to-text and text-to-speech models and runtimes, with measurements on the owner's reference machine (see EPIC-067 for the machine and its identity).
- Decide the privacy and consent model before any capture is built.
- Compare building the speech path on an existing runtime against a simpler approach (system speech APIs, where available, as an optional layer).
- Plan the first use: spoken answers in rehearsal, graded by the existing choice and typed-answer rules (EPIC-072 and the rehearsal grading).

## Acceptance Criteria
- [ ] A research note compares local speech models on accuracy, size, memory, latency, licence and offline use
- [ ] The privacy and consent position is stated
- [ ] A build-versus-buy comparison is recorded (P10)
- [ ] A strategy sets out where voice fits and how failures are handled
- [ ] Follow-up stories are listed under Decisions, each with a benchmark plan

## Design Notes
- Speech models are a hard general problem; a runtime crate is acceptable per P10, and the choice is recorded in the note.
- Model downloads follow the wizard's rules (size and source shown, a click, checksums, retry).

## Dependencies
- EPIC-067 (the reference machine and benchmark rules) for any latency figure.

## Decisions
_None yet._

## Open Questions
- Which languages must be supported first?
- Is an always-listening mode in scope, or only push-to-talk?
