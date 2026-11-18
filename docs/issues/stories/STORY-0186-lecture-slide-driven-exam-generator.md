---
id: STORY-0186
type: story
title: "Lecture Slide and Script-Driven Exam Generation Engine via Local SLM"
status: backlog
epic: EPIC-063
parents: ['EPIC-063']
estimate: 5
labels: [m14, agent, pdf]
acceptance_criteria:
  - 'Curriculum chunking engine extracting core themes, definitions, and formulas from lecture PDFs or Markdown notes'
  - 'Multi-stage SLM generation pipeline producing balanced exam question papers with sample answers and scoring rubrics'
  - 'Validation filter rejecting hallucinated or out-of-syllabus items'
---

## Context
Creating high-quality mock exams by hand is laborious. By leveraging StudyFrog's PDF text extraction and local SLM pipeline, the application can synthesize full, realistic mock exam papers directly from lecture slides or course transcripts.

## Description
Develop the automated exam generation pipeline in `studyfrog-agent`:
- Extract content sections from lecture PDFs (via `studyfrog-pdf`) or course Markdown files.
- Chunk text by module topics (e.g., "Klassische Testtheorie", "Gütekriterien", "Normierung").
- Prompt the local SLM to generate a balanced mock exam:
  - 40% Conceptual recall questions.
  - 30% Methodological case scenarios.
  - 30% Calculation exercises with explicit reference solutions and point schemes.
- Parse and validate the output into a structured `MockExam` instance ready for execution.

## Acceptance Criteria
- [ ] Curriculum chunking engine extracting core themes, definitions, and formulas from lecture PDFs or Markdown notes
- [ ] Multi-stage SLM generation pipeline producing balanced exam question papers with sample answers and scoring rubrics
- [ ] Validation filter rejecting hallucinated or out-of-syllabus items

## Design Notes
Ensure generation handles large documents cleanly through staged prompting without token overflow. Implement fallback generation templates when offline.

## Dependencies
EPIC-063, STORY-0185

## Open Questions
None.
