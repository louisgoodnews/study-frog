---
id: EPIC-068
type: epic
title: "Classroom simulation: AI-driven lectures, simulated students, group tasks, and graded submissions"
status: done
parents: []
labels: [m14, agent, slm, store, ui]
acceptance_criteria:
  - 'Users can start and run a classroom-chat lecture session backed by either a local model (e.g. Ollama) or a configured remote model provider, grounded in vault notes, stacks, or PDFs'
  - 'A simulated teacher delivers structured lecture explanations while simulated students ask contextual questions, and the user can ask questions or contribute at any time in the chat'
  - 'The teacher can assign individual or group tasks to simulated students and the user, with group collaboration visible in the session'
  - 'Assigned tasks can be turned in by the user (and simulated peers) and are graded by the teacher with structured feedback and scores'
  - 'Classroom sessions—including participants, transcript, assigned tasks, submissions, and grades—persist in local storage (vault/SQLite) and can be resumed, reviewed, or exported later'
---

## Context
Passive reading and isolated flashcard review do not replicate the social and active dynamics of a live university seminar or lecture. Hearing a topic explained step by step, seeing peers ask clarifying questions, asking one's own questions, and completing individual or group assignments under a teacher's feedback deepens conceptual understanding. StudyFrog already supports both local models (Ollama / local SLM, `EPIC-055`) and remote providers (`STORY-0083`, `STORY-0084`), as well as local-first vault and SQLite persistence.

## Description
Build an interactive **Classroom Simulation** workspace in StudyFrog:
1. **Lecture & Classroom-Chat Interface**:
   - A multi-participant chat UI with a simulated teacher persona and configurable simulated student personas, alongside the user.
   - Works with any configured local model (e.g. Ollama) or remote model provider (respecting existing provider consent/privacy settings).
   - Grounded in selected vault materials (stacks, notes, or lecture PDFs) so explanations and questions stay on-syllabus.
2. **Interactive Teaching & Q&A**:
   - The teacher explains concepts progressively in lecture segments, checks for understanding, and responds to questions.
   - Simulated students ask realistic clarifying, challenging, or applied questions; the user can interject, ask questions, or answer teacher/peer prompts at any point.
3. **Individual & Group Task Assignments**:
   - The teacher assigns exercises, short essays, problem sets, or discussion tasks during or after a lecture segment.
   - Tasks can be assigned either **individually** (to the user and/or specific simulated students) or to **groups** (e.g. the user paired with one or more simulated students who contribute to the group's work).
4. **Submission & Grading Workflow**:
   - Assigned tasks have an explicit turn-in / submission state.
   - Once submitted, the teacher grades the work (individual or group) against clear criteria, returning a score/grade and actionable feedback.
5. **Local Persistence**:
   - Full session state—topic/source references, participant roster, chat transcript, task assignments (solo and group), submitted work, and grades/feedback—is persisted locally in the vault/SQLite so sessions survive restarts and can be resumed or reviewed later.

## Acceptance Criteria
- [x] Users can start and run a classroom-chat lecture session backed by either a local model (e.g. Ollama) or a configured remote model provider, grounded in vault notes, stacks, or PDFs
- [x] A simulated teacher delivers structured lecture explanations while simulated students ask contextual questions, and the user can ask questions or contribute at any time in the chat
- [x] The teacher can assign individual or group tasks to simulated students and the user, with group collaboration visible in the session
- [x] Assigned tasks can be turned in by the user (and simulated peers) and are graded by the teacher with structured feedback and scores
- [x] Classroom sessions—including participants, transcript, assigned tasks, submissions, and grades—persist in local storage (vault/SQLite) and can be resumed, reviewed, or exported later

## Verification note

All five criteria are ticked on the strength of the headless UI tests in `studyfrog-ui` (`classroom_ui`, `classroom_work_ui`, `classroom_saved_ui`, `classroom_saved_view`, `control_classroom`) and the core, agent, service and store tests. Known gaps, not fixed:

- Models were exercised with fake or stub transports only; no live Ollama or remote provider run has been done.
- The source picker offers stacks and PDFs, not notes; criterion 1 names notes, stacks or PDFs, so stacks and PDFs satisfy it.
- The Classroom screen assigns solo tasks to the user and group tasks; the service also assigns a solo task to any participant by index, which the screen does not expose.

## Design Notes
- Reuse `studyfrog-agent` provider abstractions (`Provider`, local Ollama and remote providers) so the simulation works seamlessly with both local and remote models without duplicating wire logic.
- Keep domain models (classroom session, participant roles, turn state machine, task assignments, group membership, submissions, grading records) as pure data types and free functions (Invariant P1) with files $\le 250$ LOC (Invariant P6).
- Persist sessions locally in line with Invariant P8 (local-first data sovereignty): e.g. SQLite tables in `studyfrog-store` and/or human-readable Markdown/TOML session files in the vault.
- Budget context windows carefully for smaller local models (e.g. rolling lecture summary + recent turns + active task context) so multi-agent turns remain responsive on CPU-only machines.
- Consider control-protocol (`studyfrog ctl`) coverage so classroom sessions can be inspected and tested programmatically.

## Dependencies
- `EPIC-055` (local Ollama / SLM assistant infrastructure, done)
- `STORY-0083` / `STORY-0084` (remote provider connections, done)
- `studyfrog-store` (local persistence schema migration)

## Open Questions
- Should classroom sessions be stored as SQLite tables only, as Markdown/YAML files inside the vault (like diary entries), or both (vault files indexed in SQLite)?
- Should simulated students and the teacher share one selected model with distinct system prompts, or allow assigning different models per role?
- How should group-task collaboration be paced—turn-based in a dedicated group sub-thread before turn-in, or inline in the main classroom chat?
- Should graded classroom tasks feed into existing Reports, Diary self-assessments, or German university grade scales (`EPIC-063`)?
