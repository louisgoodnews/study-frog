---
id: STORY-0211
type: story
title: "Classroom-chat UI, session browser, and assignment submission workspace"
status: backlog
epic: EPIC-068
parents: ['EPIC-068']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - 'Users can configure and launch a classroom session (source material, local/remote model, simulated student count/personas) from a dedicated Classroom screen'
  - 'The classroom-chat interface clearly distinguishes teacher explanations, simulated student questions, and user messages, with controls to advance the lecture or ask a question'
  - 'An assignments panel/drawer displays solo and group tasks, supports group discussion and explicit turn-in, and renders teacher grades and feedback'
  - 'Users can browse, resume, review, and delete persisted classroom sessions, covered by headless UI tests and control-protocol inspection'
---

## Context
To make classroom simulation approachable and distraction-free, StudyFrog needs a dedicated UI workspace that brings together the lecture chat stream, participant roster, active solo/group assignments, grade feedback, and saved session history while following StudyFrog's calm theme and accessibility standards.

## Description
Build the Classroom screen in `studyfrog-ui`:
- **Session Setup & Browser**: Create a new session (pick stack/notes/PDF, model provider, simulated student personas) or resume/review/delete persisted sessions from local storage.
- **Classroom-Chat View**: Role-badged message stream (teacher, simulated students, user) using `palette_of(ui)` tokens, streaming/progress indicator during model turns, user input bar, and "Continue lecture" / "Ask simulated student" / "Assign task" actions.
- **Assignments & Grading Panel**: List of assigned solo and group tasks with status badges (`assigned`, `in_progress`, `submitted`, `graded`), group collaboration sub-thread, submission editor with Turn In button, and graded feedback display.
- **Verification & Control Protocol**: Headless `egui_kittest` tests and `studyfrog ctl` navigation/state coverage.

## Acceptance Criteria
- [ ] Users can configure and launch a classroom session (source material, local/remote model, simulated student count/personas) from a dedicated Classroom screen
- [ ] The classroom-chat interface clearly distinguishes teacher explanations, simulated student questions, and user messages, with controls to advance the lecture or ask a question
- [ ] An assignments panel/drawer displays solo and group tasks, supports group discussion and explicit turn-in, and renders teacher grades and feedback
- [ ] Users can browse, resume, review, and delete persisted classroom sessions, covered by headless UI tests and control-protocol inspection

## Design Notes
- Follow theme integrity (only `palette_of(ui)` tokens, never raw RGB literals), accessible widget names, and the 250-LOC file limit (Invariant P6).
- Run all model calls asynchronously on the background worker thread via `Effect` so the UI remains responsive during local-model inference.

## Dependencies
- `EPIC-068`, `STORY-0208`, `STORY-0209`, `STORY-0210`
- Child tasks: `TASK-0293`, `TASK-0294`, `TASK-0295`

## Open Questions
- Where the Classroom entry should sit in the left sidebar (under *Study* next to *Rehearse* or under *Insight* next to *Assistant*).
