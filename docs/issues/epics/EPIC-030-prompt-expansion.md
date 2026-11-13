---
id: EPIC-030
type: epic
title: "Prompt expansion"
status: done
parents: []
labels: [m12, agent, docs, ui]
acceptance_criteria:
  - "The prompt library is expanded with comprehensive interactive and analytical prompt templates (explanation, examination, Socratic dialogue, and concept mapping)"
  - "Prompts support multi-level depth parameters and structured feedback cycles for user study sessions"
  - "The prompt library UI is categorized and supports running interactive prompts directly with connected AI assistants"
---

## Context
In StudyFrog 0.4.0 (EPIC-026, STORY-0077), a basic prompt library was introduced featuring six static prompts designed primarily for generating importable CSV batches (flashcards, questions, stacks, notes, summaries, and critiques) from attached files. While effective for content generation, learners frequently require deeper cognitive engagement directly with AI models: testing their understanding through guided oral/written exams, receiving calibrated explanations at varying levels of abstraction, and working through complex conceptual materials via Socratic inquiry.

## Description
Expand StudyFrog's prompt library and interactive prompting capabilities in version 0.5.0:
1. Multi-level Explanation Prompts:
   - Provide prompts that explain an attached document or concept at varying difficulty/abstraction tiers (e.g., intuitive analogy/ELI5, undergraduate core principles, and rigorous technical deep-dive).
2. Interactive Examination & Formative Feedback Prompts:
   - Instruct AI models to conduct oral or written diagnostic exams on an attached file, assess the learner's answers, identify knowledge gaps, and provide actionable feedback.
3. Socratic Dialogue & Active Inquiry Prompts:
   - Prompts that act as guided Socratic tutors, using sequential inquiry rather than direct exposition to help learners discover conclusions and resolve misunderstandings.
4. Concept Mapping & Prerequisite Analysis Prompts:
   - Prompts that extract mental models, contrast matrices, and prerequisite concept graphs from dense texts.
5. Prompt Library UX & Parameterization:
   - Organize the prompt library into intuitive functional categories (Content Generation, Interactive Tutoring & Examination, Comprehension & Analysis) and support interactive parameter selection in the Assistant UI.

## Acceptance Criteria
- [x] The prompt library is expanded with comprehensive interactive and analytical prompt templates (explanation, examination, Socratic dialogue, and concept mapping)
- [x] Prompts support multi-level depth parameters and structured feedback cycles for user study sessions
- [x] The prompt library UI is categorized and supports running interactive prompts directly with connected AI assistants

## Design Notes
Child issues planned for version 0.5.0:
- STORY-0088: Multi-level file explanation prompt
- STORY-0089: Interactive file examination and feedback prompt
- STORY-0090: Socratic dialogue and active inquiry tutor prompt
- STORY-0091: Concept mapping and prerequisite analysis prompt
- STORY-0092: Prompt library categorization and parameterized execution UI
- BUG-0011: Importer only accepts TSV format and fails on CSV prompt outputs

Crate touchpoints:
- `docs/prompts/`: Add new template markdown files (`explain.md`, `exam.md`, `socratic.md`, `concept_map.md`).
- `crates/studyfrog-service/src/prompts.rs`: Expand `PROMPTS` registry, support prompt categories, and implement prompt parameter interpolation.
- `crates/studyfrog-ui/src/view/agent_prompts.rs`: Implement categorized navigation tabs/pills, parameter sliders/dropdowns, and enriched preview/copy actions.
- Release target: version 0.5.0 (Phase M12).

## Dependencies
EPIC-026 (STORY-0077 prompt foundation) and EPIC-028 (model integration).

## Open Questions
None.
