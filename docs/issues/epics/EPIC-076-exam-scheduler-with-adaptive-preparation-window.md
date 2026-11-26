---
id: EPIC-076
type: epic
title: "Exam scheduler: automatically schedule exam-linked stacks and items within a user-set or stats-proposed week window"
status: backlog
parents: []
labels: [m14, calendar, srs, report, ui]
acceptance_criteria:
  - 'Users can create an exam with a target date and attach associated vault items to it (whole stacks, individual flashcards, questions, and notes)'
  - 'Users can either set a preparation window in weeks themselves (e.g. N weeks before the exam date) or accept a StudyFrog-proposed week window calculated from vault stats (item count, familiarity, FSRS stability/retrievability, past performance, and study pace)'
  - 'StudyFrog automatically schedules the exam''s associated items across the chosen preparation window—prioritizing unfamiliar and weak items early, pacing reviews toward peak retention on exam day, and populating calendar study sessions / planned review blocks'
  - 'Users can preview the generated schedule (daily workload, coverage by week, projected readiness) before applying, and re-balance the schedule when items are added/removed or days are missed'
  - 'Exam-to-item links, chosen window settings, and generated schedules persist locally in SQLite and integrate cleanly with the Calendar, Home, and Rehearsal views'
---

## Context
StudyFrog already tracks calendar exam events (`studyfrog-calendar`), FSRS/SM-2 memory states (`studyfrog-srs`), familiarity and cognitive telemetry (`EPIC-060`), and workload forecasting (`STORY-0105` Exam Deadline Pacer). However, when a user adds an exam, they still have to manually decide when to start studying and manually plan sessions for the relevant stacks, cards, questions, and notes. Automatically scheduling exam-linked items within a configurable or evidence-based preparation window closes the loop between exam planning and daily active recall.

## Description
Build an **Exam Scheduler** spanning `studyfrog-core`, `studyfrog-store`, `studyfrog-srs`, `studyfrog-calendar`, `studyfrog-service`, and `studyfrog-ui`:
1. **Exam & Linked Material Model (`studyfrog-core`, `studyfrog-store`)**:
   - Extend exam records to link arbitrary vault material: one or more stacks (including sub-stacks) and/or individual notes, flashcards, and questions.
   - Persist the exam's target date, preparation window mode (`UserWeeks(u16)` or `Proposed`), and generated schedule entries in SQLite.
2. **Adaptive Preparation-Window Estimator (`studyfrog-srs` / `studyfrog-report`)**:
   - Pure free function estimating the recommended preparation window in weeks from the linked items' current state and user stats:
     - Total item count and share of unstudied/new items.
     - Current familiarity distribution, FSRS stability/difficulty, and lapse/leech rate.
     - Recent review accuracy, reading/answer velocity, and configured daily review capacity.
   - Returns both the proposed week count and a short human-readable explanation of the factors driving the estimate.
3. **Automatic Windowed Item Scheduling (`studyfrog-calendar`, `studyfrog-srs`)**:
   - Pure scheduler distributing all exam-linked items across `[exam_date - N weeks, exam_date - 1 day]`:
     - Introduces new and lowest-familiarity items early enough for multiple spaced passes before exam day.
     - Respects daily review budgets and timetable busy slots where available, leaving a final consolidation/buffer period right before the exam.
     - Supports deterministic **re-balancing** if the user adds/removes items, edits the window, or misses planned days.
4. **Exam Scheduler UI & Calendar/Rehearsal Integration (`studyfrog-ui`)**:
   - Exam creation/editing flow letting users attach stacks/cards/questions/notes, toggle between **Custom weeks** and **StudyFrog proposed (N weeks)** (with stats breakdown), preview the weekly/daily workload, and apply the schedule.
   - Scheduled blocks appear in Calendar (Month/Week/Day views), Home ("Coming up" / "Due now"), and can launch a scoped rehearsal run for that day's exam items in one click.

## Acceptance Criteria
- [ ] Users can create an exam with a target date and attach associated vault items to it (whole stacks, individual flashcards, questions, and notes)
- [ ] Users can either set a preparation window in weeks themselves (e.g. `N` weeks before the exam date) or accept a StudyFrog-proposed week window calculated from vault stats (item count, familiarity, FSRS stability/retrievability, past performance, and study pace)
- [ ] StudyFrog automatically schedules the exam's associated items across the chosen preparation window—prioritizing unfamiliar and weak items early, pacing reviews toward peak retention on exam day, and populating calendar study sessions / planned review blocks
- [ ] Users can preview the generated schedule (daily workload, coverage by week, projected readiness) before applying, and re-balance the schedule when items are added/removed or days are missed
- [ ] Exam-to-item links, chosen window settings, and generated schedules persist locally in SQLite and integrate cleanly with the Calendar, Home, and Rehearsal views

## Design Notes
- Reuse existing `studyfrog-calendar::review_plan`, `studyfrog-report` familiarity/velocity telemetry (`EPIC-060`), and `studyfrog-srs` FSRS simulations (`STORY-0105`) rather than duplicating scheduling math.
- Keep all window-estimation and scheduling algorithms pure free functions (Invariant P1) with deterministic property/unit tests and files $\le 250$ LOC (Invariant P6).
- Respect Invariant P8 (local-first data sovereignty) and keep existing SRS scheduling non-destructive unless the user explicitly applies an exam schedule.

## Dependencies
- Existing Calendar & review planner (`EPIC-010`, `EPIC-044`, `EPIC-056`, done)
- Familiarity, cognitive profiling & FSRS forecasting (`EPIC-034`, `EPIC-060`, done)

## Open Questions
- Should exam-scheduled items temporarily override per-stack daily new-card caps when an exam window is tight, or warn the user that the requested window is shorter than their daily budget allows?
- Should multiple overlapping exams share a single combined daily workload budget so two exams in the same week do not double-book the user?
