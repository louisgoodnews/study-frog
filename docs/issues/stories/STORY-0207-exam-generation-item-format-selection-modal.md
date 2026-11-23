---
id: STORY-0207
type: story
title: "Interactive modal to choose item formats during mock exam generation"
status: done
epic: EPIC-063
parents: ['EPIC-063']
estimate: 5
labels: [m14, rehearse, agent, ui]
acceptance_criteria:
  - 'Launching mock exam generation (from notes or lecture PDFs) opens an interactive configuration modal where the user selects which item formats to include (e.g. open question, fill in the gaps / cloze, single select, multiple select, calculation, case scenario)'
  - 'The modal lets the user set the desired format mix (counts or weights per selected format) and total question count, with sensible defaults and validation preventing an empty selection'
  - 'The exam generation pipeline (both local/remote model prompting and offline template fallback) respects the selected item formats and produces structured questions matching each chosen format'
  - 'The mock exam runner and grading/review views render, accept answers for, and grade every supported format (free text, gap fills, single-choice radio selection, multi-choice checkbox selection)'
  - 'Selected format preferences are remembered for the session and covered by headless UI and service tests'
---

## Context
Mock exam generation (`EPIC-063`, `STORY-0186`) currently uses a fixed question mix (conceptual recall, case scenarios, and calculations) with free-text answers only. Real university exams mix formats—open-ended questions, fill-in-the-blank / cloze items, single-choice, multiple-select, calculations, and case vignettes—and students want control over which formats appear when generating a practice exam.

## Description
Add an interactive **Exam Generation Format Modal** before generating a mock exam from stack notes or lecture PDFs:
1. **Interactive Format Selection Modal (`studyfrog-ui`)**:
   - Triggered when clicking **Mock exam** or **Mock exam from a PDF** in rehearse setup.
   - Displays toggleable item formats: **Open question**, **Fill in the gaps (cloze)**, **Single select (single choice)**, **Multiple select (multi choice)**, **Calculation**, and **Case scenario**.
   - Allows specifying per-format counts or proportions (plus total question count) with a live summary and validation so at least one format is selected.
2. **Format-Aware Generation (`studyfrog-rehearsal`, `studyfrog-agent`, `studyfrog-service`)**:
   - Extend the exam generation request and structured question model to carry the requested item formats and per-format payloads (e.g. cloze blanks, option lists with single/multiple correct keys).
   - Update SLM prompts, JSON parsing, validation filter, and offline template fallbacks to generate and validate each requested format.
3. **Interactive Answering & Grading (`studyfrog-ui`, `studyfrog-rehearsal`)**:
   - Render format-appropriate controls during the timed exam run (text area for open questions, inline gap inputs for fill-in-the-gaps, radio buttons for single select, checkboxes for multiple select).
   - Grade objective formats (single select, multiple select, exact/normalized gap fills) deterministically alongside SLM/keyword grading for open questions and case scenarios.

## Acceptance Criteria
- [x] Launching mock exam generation (from notes or lecture PDFs) opens an interactive configuration modal where the user selects which item formats to include (e.g. open question, fill in the gaps / cloze, single select, multiple select, calculation, case scenario)
- [x] The modal lets the user set the desired format mix (counts or weights per selected format) and total question count, with sensible defaults and validation preventing an empty selection
- [x] The exam generation pipeline (both local/remote model prompting and offline template fallback) respects the selected item formats and produces structured questions matching each chosen format
- [x] The mock exam runner and grading/review views render, accept answers for, and grade every supported format (free text, gap fills, single-choice radio selection, multi-choice checkbox selection)
- [x] Selected format preferences are remembered for the session and covered by headless UI and service tests

## Design Notes
- Reuse existing cloze (`studyfrog-md` / `STORY-0108`) and case-study choice/option primitives where they fit cleanly, keeping all logic in pure free functions (Invariant P1) and files $\le 250$ LOC (Invariant P6).
- For multiple select, define explicit partial-credit or all-or-nothing scoring rules in `studyfrog-rehearsal::grade` and document them in the graded review view.
- Ensure keyboard navigation and accessible widget labels (`ui tree` / `ui click`) work inside the modal and for every answer control.

## Dependencies
- `EPIC-063`, `STORY-0185`, `STORY-0186`, `STORY-0187`

## Open Questions
- Should multiple-select grading use all-or-nothing scoring or partial credit with a penalty for wrong selections (standard German *Mehrfachauswahl* rules)?
- Should the modal also expose exam duration / pass-mark overrides alongside item formats?

## Implementation (2026-11-23)

- **`studyfrog-rehearsal`**: Added `exam_format` / `exam_format_mix` (`ExamItemFormat` 6 variants, `MixMode`, `ExamFormatMix`, `FormatPlan`, validation & Hamilton largest-remainder resolution, `Part A`..`Part F` headings), `curriculum_choices` / `curriculum_gaps` offline synthesizers, and `exam_answer` / `exam_answer_grade` (`ExamAnswer`, `MULTI_SELECT_RULE` net partial credit `max(0, hits - wrong) / correct_count`, normalized `grade_fill_in_gaps`, `grade_single_select`, `grade_multiple_select`).
- **`studyfrog-agent` & `studyfrog-service`**: Added `FormatCounts`, extended `RawQuestion`, `chunk_system_for_formats`, `exam_gen_judge` (`studyfrog_md::cloze::cloze_spans` normalization), `generate_with_mix`, and format-aware `exam_markdown`.
- **`studyfrog-ui`**: Added `Modal::MockExamFormat`, session-persistent `Rehearse::mock_format_prefs`, format-specific answering controls (textareas, inline Tab-navigable cloze gaps, single-select radios, multiple-select checkboxes), `By format` breakdown, and explicit `MULTI_SELECT_RULE` display in graded review. Covered by `studyfrog-rehearsal`, `studyfrog-agent`, `studyfrog-service`, and `egui_kittest` suites.
