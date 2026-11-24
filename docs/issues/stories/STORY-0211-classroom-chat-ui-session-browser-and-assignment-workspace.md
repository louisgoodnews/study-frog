---
id: STORY-0211
type: story
title: "Classroom-chat UI, session browser, and assignment submission workspace"
status: done
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
- [x] Users can configure and launch a classroom session (source material, local/remote model, simulated student count/personas) from a dedicated Classroom screen
- [x] The classroom-chat interface clearly distinguishes teacher explanations, simulated student questions, and user messages, with controls to advance the lecture or ask a question
- [x] An assignments panel/drawer displays solo and group tasks, supports group discussion and explicit turn-in, and renders teacher grades and feedback
- [x] Users can browse, resume, review, and delete persisted classroom sessions, covered by headless UI tests and control-protocol inspection

## Design Notes
- Follow theme integrity (only `palette_of(ui)` tokens, never raw RGB literals), accessible widget names, and the 250-LOC file limit (Invariant P6).
- Run all model calls asynchronously on the background worker thread via `Effect` so the UI remains responsive during local-model inference.

## Dependencies
- `EPIC-068`, `STORY-0208`, `STORY-0209`, `STORY-0210`
- Child tasks: `TASK-0293`, `TASK-0294`, `TASK-0295`

## Open Questions
- Where the Classroom entry should sit in the left sidebar (under *Study* next to *Rehearse* or under *Insight* next to *Assistant*).

## Verification note

All four criteria are ticked on the strength of the headless UI tests in `studyfrog-ui` (`classroom_ui`, `classroom_saved_ui`, `classroom_saved_view`, `classroom_work_ui`, `classroom_work_view`, `control_classroom`), the service and store tests, and the agent tests. Known gaps, not fixed:

- Models were exercised with fake or stub transports only; no live Ollama or remote provider run has been done.
- Source material is stacks and PDFs only; notes are not offered as a source.
- Personas are fixed presets (count 0 to 3); custom personas are not offered.
- The assignments panel sets solo tasks for the user ("Assign task") and groups of two ("Assign group task"); a solo task for a simulated student is not offered in the UI.
- The rolling lecture summary is not persisted (see TASK-0290).
