---
id: STORY-0089
type: story
title: "Interactive file examination and feedback prompt"
status: done
epic: EPIC-030
parents: [EPIC-030]
estimate: 3
labels: [m12, agent, docs]
acceptance_criteria:
  - "A standard examination and feedback prompt exists in docs/prompts/exam.md and compiles into studyfrog-service"
  - "The prompt instructs the AI agent to quiz the learner on the attached file, evaluate answers against source material, and provide diagnostic feedback"
  - "The feedback highlights accuracy, misconceptions, and suggested follow-up flashcard or practice questions"
---

## Context
Active recall and testing are proven to be the most effective study strategies. Beyond passive flashcard review, learners benefit from an interactive examiner that asks open-ended questions based on their documents (PDFs, notes, or code), evaluates their typed answers, identifies subtle gaps, and offers targeted feedback.

## Description
Author an interactive examination prompt for StudyFrog's prompt library:
1. Prompt Template (`docs/prompts/exam.md`):
   - Instructs the AI agent to act as a rigorous yet encouraging oral examiner specializing in the attached document's subject.
   - Poses 3 to 5 challenging comprehension and application questions (avoiding trivial trivia or direct quote regurgitation).
   - Invites the user to answer questions sequentially or all at once.
2. Evaluation Rubric & Feedback Structure:
   - For each user response, evaluates:
     - Accuracy: What the user got right based strictly on the source material.
     - Omissions & Misconceptions: Crucial points missed or inaccurately described.
     - Model Answer & Elaboration: Concise reference answer explaining the ideal synthesis.
   - Recommends 2-3 specific flashcard or practice questions for the learner to add to their vault stacks to reinforce weak spots.
3. Offline & Online Compatibility:
   - Functions both when copied into external chatbot sessions (ChatGPT, Claude, local LLMs) and when run within StudyFrog's connected assistant dialogue.

## Acceptance Criteria
- [x] A standard examination and feedback prompt exists in docs/prompts/exam.md and compiles into studyfrog-service
- [x] The prompt instructs the AI agent to quiz the learner on the attached file, evaluate answers against source material, and provide diagnostic feedback
- [x] The feedback highlights accuracy, misconceptions, and suggested follow-up flashcard or practice questions

## Design Notes
- Template file: `docs/prompts/exam.md`.
- Registry: `crates/studyfrog-service/src/prompts.rs`.
- Compatible with existing `ATTACH_LIMIT` (60,000 characters) and markdown file clipping in `studyfrog-service`.
- Release target: version 0.5.0 (Phase M12).

## Dependencies
EPIC-030.

## Open Questions
None.
