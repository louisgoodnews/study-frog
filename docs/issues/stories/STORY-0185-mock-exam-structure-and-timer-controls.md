---
id: STORY-0185
type: story
title: "Mock Exam Structure, Weighted Point Allocations, and Strict Timer Controls"
status: done
epic: EPIC-063
parents: ['EPIC-063']
estimate: 3
labels: [m14, rehearse]
acceptance_criteria:
  - 'Domain model for MockExamDefinition, ExamSection, and QuestionPoints in crates/studyfrog-rehearsal'
  - 'Strict test mode state machine tracking elapsed time against total exam budget with auto-submit on timeout'
  - 'Question navigation allowing jumping between unanswered questions and flagging items for review'
---

## Context
A realistic exam simulation requires formal structure: different questions carry different point weights (e.g. 2 points vs 8 points), an overall exam time limit (e.g. 60 or 90 minutes) is strictly enforced, and students need the ability to navigate between questions before final submission.

## Description
Develop the mock exam domain model in `studyfrog-rehearsal`:
- Define `MockExam`, `ExamQuestion`, `QuestionWeight`, and `ExamProgress`.
- Support section-based point budgeting:
  - Part A: Short-answer definitions (1-2 pts each).
  - Part B: Psychometric calculations (3-5 pts each).
  - Part C: Case scenario applications (5-10 pts each).
- Implement navigation logic: jump to question, mark for review, review overview screen showing answered vs unanswered questions.
- Enforce strict timer controls with optional warning alerts (10 minutes remaining, 2 minutes remaining) and automatic submission when the clock expires.

## Acceptance Criteria
- [x] Domain model for MockExamDefinition, ExamSection, and QuestionPoints in crates/studyfrog-rehearsal
- [x] Strict test mode state machine tracking elapsed time against total exam budget with auto-submit on timeout
- [x] Question navigation allowing jumping between unanswered questions and flagging items for review

## Design Notes
Keep all operations free functions with zero inherent `impl` blocks (Invariant P1). Thoroughly test timer expiration, boundary points, and navigation state transitions.

## Dependencies
EPIC-063

## Implementation (2026-11-21)
`studyfrog-rehearsal::mock_exam` (model and validation), `mock_nav` (jump, flag, next unanswered, review overview), `mock_timer` (10-minute and 2-minute alerts, expiry). Tests: `tests/mock_exam.rs`. The exam screens are part of STORY-0187.

## Open Questions
None.
