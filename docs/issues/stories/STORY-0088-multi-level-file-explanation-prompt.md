---
id: STORY-0088
type: story
title: "Multi-level file explanation prompt"
status: done
epic: EPIC-030
parents: [EPIC-030]
estimate: 3
labels: [m12, agent, docs]
acceptance_criteria:
  - "A standard explanation prompt exists in docs/prompts/explain.md and compiles into studyfrog-service"
  - "The prompt instructs the AI to explain an attached file at three selectable levels of abstraction: Intuitive Analogy (ELI5), Core Concepts, and Technical Deep-Dive"
  - "The generated explanation produces clear Markdown notes structured for StudyFrog import and immediate reading"
---

## Context
When tackling difficult textbooks, research papers, or complex source code, learners often get stuck on jargon or dense prose. Having an AI explain the document at an appropriate cognitive level—whether breaking down abstract math into everyday analogies or unpacking the nuanced architectural tradeoffs of a codebase—accelerates comprehension.

## Description
Create a multi-level explanation prompt template and integrate it into StudyFrog:
1. Prompt Template (`docs/prompts/explain.md`):
   - Instructs the AI agent to examine the attached document and provide explanations across three distinct levels of depth:
     - Level 1 (Intuitive Analogy / ELI5): Focuses on the core intuition using vivid analogies, plain language, and zero assumed prerequisite jargon.
     - Level 2 (Core Concepts & Mechanisms): Explains the main principles, standard terminology, component interactions, and practical significance.
     - Level 3 (Technical Deep-Dive): Details formal definitions, edge cases, underlying mechanics, mathematical derivations, or low-level implementation details.
2. Output Formatting:
   - Structures the output either as clean Markdown notes with heading hierarchies suitable for direct reading in the Assistant, or as an importable note block with tags (`explain/intuitive`, `explain/core`, `explain/technical`).
3. Worked Example & Verification:
   - Include a worked example in the prompt file that passes automated syntax validation and import testing.

## Acceptance Criteria
- [x] A standard explanation prompt exists in docs/prompts/explain.md and compiles into studyfrog-service
- [x] The prompt instructs the AI to explain an attached file at three selectable levels of abstraction: Intuitive Analogy (ELI5), Core Concepts, and Technical Deep-Dive
- [x] The generated explanation produces clear Markdown notes structured for StudyFrog import and immediate reading

## Design Notes
- Template file: `docs/prompts/explain.md`.
- Code touchpoint: `crates/studyfrog-service/src/prompts.rs`, registering `prompt!("explain")`.
- Ensure worked example conforms with `_layout.md`, `_files.md`, and `_answer.md` interpolation rules.
- Release target: version 0.5.0 (Phase M12).

## Dependencies
EPIC-030.

## Open Questions
None.
