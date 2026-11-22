---
id: STORY-0179
type: story
title: "Scenario-Based Diagnostic Case Study Item Schema and Domain Model"
status: done
epic: EPIC-061
parents: ['EPIC-061']
estimate: 3
labels: [m14, rehearse, store]
acceptance_criteria:
  - 'Pure Rust data model for CaseStudyItem with vignette, prompt, expected decision, and rubric criteria'
  - 'Markdown frontmatter and storage serialization in vaults under existing stack hierarchies'
  - 'Validation functions ensuring well-formed rubrics and scenario length limits'
---

## Context
Case-study learning requires a dedicated domain schema that captures an authentic clinical/diagnostic scenario, the expected procedural choice, and a breakdown of grading rubric criteria.

## Description
Implement the core data model in `studyfrog-core` / `studyfrog-rehearsal`:
- Define `CaseStudyItem` with:
  - `vignette`: The rich scenario description (e.g. organizational setting, target construct, constraints).
  - `decision_prompt`: What decision the student must make.
  - `expected_choice`: The reference choice or accepted procedure list.
  - `rubric_criteria`: List of essential conceptual points required in the justification (e.g. "addresses fabbing risk", "cites criterion validity").
- Support serialization to Markdown card format with extended frontmatter.
- Implement pure validation functions.

## Acceptance Criteria
- [x] Pure Rust data model for CaseStudyItem with vignette, prompt, expected decision, and rubric criteria
- [x] Markdown frontmatter and storage serialization in vaults under existing stack hierarchies
- [x] Validation functions ensuring well-formed rubrics and scenario length limits

## Design Notes
Strictly adhere to Invariant P1 (zero inherent `impl` blocks). Maintain backward compatibility with existing card readers.

## Dependencies
EPIC-061

## Implementation (2026-11-21)
`studyfrog-rehearsal::case_study`: the model (vignette, decision, options, expected choices, rubric), validation with the length limits, and the order-free choice check. Tests: `tests/case_study.rs`. Vault storage: a case is a question file (`studyfrog-service::case_codec`, pure): the vignette is the prompt, the expected choices are the `accepted` list, the rubric is the `rubric` list, and the decision and options are extra front-matter lines (`decision`, `options`, separated by ` | `). Plain readers still see a question with its rubric. `case_items::create_case_study` writes the file and `load_case_studies` reads the valid cases of a stack back; plain questions and malformed cases are skipped. Tests: `studyfrog-service/tests/case_items.rs` and the codec's unit tests.

Audit (2026-11-22): all three criteria MET. (1) The model is `CaseStudyItem` (vignette, decision_prompt, options, expected_choices, rubric_criteria) with no inherent `impl`; the case view renders its fields (`/tmp/claude-0/audit/g3/02-case-list.png`). (2) The vault file `diag/case-screen.md` (front matter `decision`, `options`, `accepted`, `rubric`, vignette as prompt) loads as "Case 1 of 1" with its decision and options (`/tmp/claude-0/audit/g3/02-case-list.png`, `/tmp/claude-0/audit/g3/21-exam-case.png`). (3) Validation: the malformed file `diag/case-bad.md` (no rubric) is skipped by the case view (`/tmp/claude-0/audit/g3/02-case-list.png`, "Case 1 of 1"); the length limits are in `validate_case_study` (code, not exercised in the UI).

## Open Questions
None.
