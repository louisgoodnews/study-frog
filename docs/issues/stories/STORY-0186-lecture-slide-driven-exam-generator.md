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
- [ ] Curriculum chunking engine extracting core themes, definitions, and formulas from lecture PDFs or Markdown notes
- [x] Multi-stage SLM generation pipeline producing balanced exam question papers with sample answers and scoring rubrics
- [x] Validation filter rejecting hallucinated or out-of-syllabus items

## Implementation (2026-11-22)
Four stages, each in the crate that owns it (the layering allows agent to depend on core only, so the composition is in the service):
1. Chunking (`studyfrog-rehearsal::curriculum`): Markdown cut at headings (code fences ignored), each topic split into pieces of at most 3,000 characters; definitions (`Term: meaning`, `X ist ...`) and formulas (lines with `=` and a digit or `$`) found in each piece. A PDF's text (`studyfrog-pdf`) becomes one general topic, since PDFs carry no headings.
2. Generation (`studyfrog-agent::exam_gen`, `studyfrog-service::exam_gen`): per chunk, the model is asked for its share of the plan (40 % recall, 30 % cases, 30 % calculations) as one JSON object; the reply is read with the existing JSON span reader. A chunk whose reply fails is noted, and templates stand in for it.
3. Validation (`curriculum_build::accept`): one line of 10 to 400 characters; 1 to 10 points; a reference answer; the topic must be a chunk's heading (out of syllabus is rejected); at least 40 % of the reference answer's significant words must appear in that topic's notes (hallucination check).
4. Balance and assembly (`curriculum_build::assemble`): the planned number of each kind, topped up from templates; sections A (recall), B (cases), C (calculations); a time limit of three minutes a point. The reference answers and rubrics come back beside the exam as `AnswerKey`s, since `ExamQuestion` has no field for them.

Offline (no `ask`), the exam is made from the notes' own definitions and formulas; its answers are the notes' lines, and the templates say so.

Wired into the app: "Mock exam" on the rehearse setup's What step makes an exam of ten questions from the notes of the stack that is open (`setup::mock_exam`, `exec::mock`, `view::mock_exam_view`). The local model writes the questions when one is selected and may see the notes (a remote one only when the user allowed it); otherwise templates make it. "Save as note" writes the exam, with its reference answers, into the stack as a note (`exam_save::save_exam`). The view says when the notes supply fewer questions than asked.

Open: the exam is shown in a view of its own, not in the Altklausur lab (no lab screen exists yet). Small local models are unmeasured, as for STORY-0200. The PDF route is tested on a real PDF (exported from a note, then read back).

Tests: `studyfrog-rehearsal` (curriculum; `tests/curriculum_build.rs`), `studyfrog-agent` (exam_gen), `studyfrog-service` (`tests/exam_gen_pdf.rs`: a note exported to a real PDF, read back and made into an exam; `tests/exam_save.rs`), `studyfrog-ui/tests/mock_exam_ui.rs`. A screenshot (`--screen mock-demo`) shows an exam made offline.

## Design Notes
Ensure generation handles large documents cleanly through staged prompting without token overflow. Implement fallback generation templates when offline.

## Dependencies
EPIC-063, STORY-0185

## Implementation notes (2026-11-22, slides and the no-model message)

Built:
- Lecture slides in the app: "Mock exam from a PDF" on the rehearse setup's What step lists the readable PDFs of the vault (the index is brought up to date first). Choosing one reads its text (`studyfrog-service::exam_save::slide_chunks`, which refuses paths outside the vault and files that are not PDFs) and makes the exam with the same generator as the notes path.
- A clear message when no model is selected: the exam view says the questions come from templates and how to select a local model. When the selected model is remote and not allowed to see the text, it says so. (The check is whether a local model is selected; the app does not test that the local runtime is running. A selected local model that fails leaves the templates to write the questions; the preview counts the questions from the model and from templates.)
- The local model is asked per chunk, as before; its questions are checked against the notes (hallucination filter) and the balance is kept.

Tests: `studyfrog-service/tests/exam_gen_pdf.rs` (a slide PDF in the vault is listed after indexing, read, made into a valid exam offline, and paths outside the vault are refused); `studyfrog-ui/tests/mock_exam_model_ui.rs` (no-model and remote notes, a local model is used, a PDF is listed and then made into an exam, a failed listing says why).

Not done or not verified:
- A real local model was not run: this container has none. The model path is tested with a fake model (`exam_gen` tests). Question quality and how small models do are unmeasured.
- The Altklausur lab screen does not exist; the exam is shown in the rehearse setup's mock-exam view.
- Slides as images (scanned PDFs with no text) give no questions; OCR is not part of this story.
- A person at a window should check the PDF list, the exam view and the notes.

## Open Questions
None.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): no local model is installed, so the model stages are NOT CHECKABLE; the template path and the no-model message were checked.
Criterion 1 MET for definitions: the PDF "Vorlesung.pdf" gave "Explain the following in your own words: Gueltigkeit" and the notes gave "Normierung legt Bezugsnormen fest..." (/tmp/claude-0/audit/g5/0186-pdf-exam.png, /tmp/claude-0/audit/g5/0186-mock-notes3.png). Formulas were not seen in an exam: the one formula note was made after the exam was generated.
Criterion 2 NOT CHECKABLE for the SLM stages (no model). The template path gives sample answers and rubric points (/tmp/claude-0/audit/g5/0187-keys.png). The no-model note is shown (/tmp/claude-0/audit/g5/0186-pdf-list.png). Rubric text is generic and the keyword grading is wrong for correct answers (BUG-0038).
Criterion 3 NOT CHECKABLE: the hallucination and out-of-syllabus filter applies to model output; with no model it does not run. Defects: BUG-0039 (PDF exam titled "untitled"), BUG-0040 (mock exam uses the open stack, not the ticked one), BUG-0041 (save error in rehearsal, intermittent).

## Verification (2026-11-22)

No local model is installed, so the model stages are NOT CHECKABLE (criteria 1 to 3 as they apply to model output). The no-model path ran: the note "No model is selected, so the questions come from templates" and 4 questions from the notes, 0 from a model (`/tmp/claude-0/verifyA/20-mock-preview.png`). The hallucination and out-of-syllabus filter applies to model output and did not run. Ticks are left as set.

Review (2026-11-22): the rehearse setup's "Mock exam from a PDF" and "Mock exam" were run in the running app (no local model; the binary of the current head; `/tmp/claude-0/review-e/`).
- Criterion 1 (chunking): definitions MET for the PDF route (`/tmp/claude-0/review-e/03-pdf-exam.png`, the question "Explain the following in your own words: Gueltigkeit" from Vorlesung.pdf). Formulas NOT MET as seen: the note "Formel" with "Standardfehler = s / Wurzel(n) = 2 / 2 = 1" gave no calculation question (`/tmp/claude-0/review-e/20-mock-stat.png`; BUG-0053). Ticked earlier; unticked by this review.
- Criterion 2 (SLM pipeline with sample answers and rubrics): NOT CHECKABLE for model quality and for the model stages (no model installed). The no-model note is shown (`/tmp/claude-0/review-e/03-pdf-exam.png`: "No model is selected, so the questions come from templates"); the template path gives reference answers and rubric lines (`/tmp/claude-0/review-e/10-result.png`). The reference for card questions repeats the card question (BUG-0052). Tick left as set.
- Criterion 3 (validation filter): NOT CHECKABLE: the filter applies to model output, which did not run. Tick left as set.
- PDF path defects: BUG-0039 still open ("Mock exam: untitled"; both PDFs listed as "untitled", `/tmp/claude-0/review-e/02-pdf-list.png`, `03-pdf-exam.png`).
- BUG-0040 (ticked stack): fixed (`/tmp/claude-0/review-e/19-mock-pair.png`: Pair ticked, Statistik open, title "Mock exam: pair").
- Status stays review: criterion 1 is open (formulas) and criteria 2 and 3 cannot be checked here, and the PDF path still shows "untitled".

