---
id: STORY-0186
type: story
title: "Lecture Slide and Script-Driven Exam Generation Engine via Local SLM"
status: review
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
- [x] Curriculum chunking engine extracting core themes, definitions, and formulas from lecture PDFs or Markdown notes
- [x] Multi-stage SLM generation pipeline producing balanced exam question papers with sample answers and scoring rubrics
- [x] Validation filter rejecting hallucinated or out-of-syllabus items

## Implementation (2026-11-22)
Four stages, each in the crate that owns it (the layering allows agent to depend on core only, so the composition is in the service):
1. Chunking (`studyfrog-rehearsal::curriculum`): Markdown cut at headings (code fences ignored), each topic split into pieces of at most 3,000 characters; definitions (`Term: meaning`, `X ist ...`) and formulas (lines with `=` and a digit or `$`) found in each piece. A PDF's text (`studyfrog-pdf`) becomes one general topic, since PDFs carry no headings.
2. Generation (`studyfrog-agent::exam_gen`, `studyfrog-service::exam_gen`): per chunk, the model is asked for its share of the plan (40 % recall, 30 % cases, 30 % calculations) as one JSON object; the reply is read with the existing JSON span reader. A chunk whose reply fails is noted, and templates stand in for it.
3. Validation (`curriculum_build::accept`): one line of 10 to 400 characters; 1 to 10 points; a reference answer; the topic must be a chunk's heading (out of syllabus is rejected); at least 40 % of the reference answer's significant words must appear in that topic's notes (hallucination check).
4. Balance and assembly (`curriculum_build::assemble`): the planned number of each kind, topped up from templates; sections A (recall), B (cases), C (calculations); a time limit of three minutes a point. The reference answers and rubrics come back beside the exam as `AnswerKey`s, since `ExamQuestion` has no field for them.

Offline (no `ask`), the exam is made from the notes' own definitions and formulas; its answers are the notes' lines, and the templates say so.

Open: nothing calls the model yet from the app (the pipeline takes the model as a closure, so an effect can supply it); the exam is not saved to the vault or shown in the Altklausur lab. The notes-to-exam path is tested with fake models; no PDF fixture exists in the repository, so the PDF route is compiled but not run on a real file. Small local models are unmeasured, as for STORY-0200.

Tests: inline in `studyfrog-rehearsal` (curriculum, curriculum_build), `studyfrog-agent` (exam_gen), `studyfrog-service` (exam_gen: offline, invented question rejected, failed chunk noted).

## Design Notes
Ensure generation handles large documents cleanly through staged prompting without token overflow. Implement fallback generation templates when offline.

## Dependencies
EPIC-063, STORY-0185

## Open Questions
None.
