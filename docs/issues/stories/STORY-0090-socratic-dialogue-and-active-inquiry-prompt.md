---
id: STORY-0090
type: story
title: Socratic dialogue and active inquiry prompt
status: done
epic: EPIC-030
parents: [EPIC-030]
estimate: 3
labels: [m12, agent, docs]
acceptance_criteria:
  - "A standard Socratic dialogue prompt exists in docs/prompts/socratic.md and compiles into studyfrog-service"
  - "The prompt enforces a question-driven dialogue where the AI leads the learner to discover key insights rather than lecturing"
  - "The AI dynamically detects misunderstandings and poses counter-examples to stimulate critical thinking"
---

## Context
When studying difficult theoretical concepts, passive reading and direct explanations can create an illusion of competence. Socratic questioning forces the student to articulate reasoning, test hypotheses, and confront counter-examples, leading to far deeper understanding and retention.

## Description
Develop a Socratic tutoring prompt for the StudyFrog library:
1. Prompt Template (`docs/prompts/socratic.md`):
   - Instructs the AI model to adopt the persona of a Socratic tutor guiding the user through the attached file.
   - Hard Rule: Never lecture or provide full paragraph explanations outright; instead, ask one focused, thought-provoking question at a time.
   - Break down complex themes into incremental discoveries.
2. Handling Learner Responses:
   - If the student answers correctly: Validate concisely and ask a follow-up question that pushes into deeper implications or edge cases.
   - If the student is partially mistaken: Acknowledge the sensible intuition, then present a concrete counter-example or leading clue to help them discover the contradiction.
   - If the student says "I don't know": Provide a simplified hint or a contrasting analogy, then ask a simpler bridging question.
3. Synthesis Note Generation:
   - At the conclusion of the dialogue, the prompt instructs the AI to summarize the student's own discovered conclusions into a concise summary note ready for the vault.

## Acceptance Criteria
- [x] A standard Socratic dialogue prompt exists in docs/prompts/socratic.md and compiles into studyfrog-service
- [x] The prompt enforces a question-driven dialogue where the AI leads the learner to discover key insights rather than lecturing
- [x] The AI dynamically detects misunderstandings and poses counter-examples to stimulate critical thinking

## Design Notes
- Template file: `docs/prompts/socratic.md`.
- Registry: `crates/studyfrog-service/src/prompts.rs`.
- Follows StudyFrog prompt style guidelines: clear instructions, concise tone, and robust error handling (`PROBLEM:` fallback).
- Release target: version 0.5.0 (Phase M12).

## Dependencies
EPIC-030.

## Open Questions
None.
