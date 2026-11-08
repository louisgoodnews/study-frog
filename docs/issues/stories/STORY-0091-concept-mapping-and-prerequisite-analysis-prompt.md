---
id: STORY-0091
type: story
title: Concept mapping and prerequisite analysis prompt
status: done
epic: EPIC-030
parents: [EPIC-030]
estimate: 3
labels: [m12, agent, docs]
acceptance_criteria:
  - "A standard concept mapping and prerequisite analysis prompt exists in docs/prompts/concept_map.md and compiles into studyfrog-service"
  - "The prompt generates prerequisite dependency lists, conceptual contrast tables, and mental model summaries from an attached file"
  - "The output imports as structured Markdown notes with wikilinks connecting related concepts"
---

## Context
Learners frequently encounter advanced material without knowing what foundational concepts they are assumed to have mastered beforehand. In addition, related concepts in a document are often confused with one another. A structured prompt that extracts prerequisite chains, mental models, and contrast matrices helps learners build coherent knowledge structures in their vaults.

## Description
Create a concept mapping and prerequisite analysis prompt for the prompt library:
1. Prompt Template (`docs/prompts/concept_map.md`):
   - Instructs the AI to analyze the attached document for conceptual structure:
     - Prerequisites: What foundational ideas, math, or tools must the reader understand before this file makes sense?
     - Core Mental Models: What are the fundamental abstractions or metaphors that explain why the subject works this way?
     - Contrast Matrix: A Markdown table contrasting easily conflated concepts (e.g. Concept A vs. Concept B, differentiating criteria, typical misconceptions).
2. Wikilink Integration:
   - Prompts the AI to format references to related terms as `[[Concept Name]]` wikilinks, allowing imported notes to mesh directly with StudyFrog's vault graph and backlinks.
3. Worked Example & Verification:
   - Include a valid CSV worked example in the prompt with note records containing tables, wikilinks, and prerequisite tags.

## Acceptance Criteria
- [x] A standard concept mapping and prerequisite analysis prompt exists in docs/prompts/concept_map.md and compiles into studyfrog-service
- [x] The prompt generates prerequisite dependency lists, conceptual contrast tables, and mental model summaries from an attached file
- [x] The output imports as structured Markdown notes with wikilinks connecting related concepts

## Design Notes
- Template file: `docs/prompts/concept_map.md`.
- Registry: `crates/studyfrog-service/src/prompts.rs`.
- Ensure output schema generates valid CSV blocks matching `crates/studyfrog-service/tests/prompts.rs`.
- Release target: version 0.5.0 (Phase M12).

## Dependencies
EPIC-030.

## Open Questions
None.
