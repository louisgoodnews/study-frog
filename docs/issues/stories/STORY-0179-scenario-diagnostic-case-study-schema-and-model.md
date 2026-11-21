---
id: STORY-0179
type: story
title: "Scenario-Based Diagnostic Case Study Item Schema and Domain Model"
status: review
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
- [ ] Pure Rust data model for CaseStudyItem with vignette, prompt, expected decision, and rubric criteria
- [ ] Markdown frontmatter and storage serialization in vaults under existing stack hierarchies
- [ ] Validation functions ensuring well-formed rubrics and scenario length limits

## Design Notes
Strictly adhere to Invariant P1 (zero inherent `impl` blocks). Maintain backward compatibility with existing card readers.

## Dependencies
EPIC-061

## Implementation (2026-11-21)
`studyfrog-rehearsal::case_study`: the model (vignette, decision, options, expected choices, rubric), validation with the length limits, and the order-free choice check. Tests: `tests/case_study.rs`. Open: Markdown front matter serialization in the vault.

## Open Questions
None.
