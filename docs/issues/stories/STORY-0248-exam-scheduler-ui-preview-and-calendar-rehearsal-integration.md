---
id: STORY-0248
type: story
title: "Exam Scheduler UI, workload preview, and Calendar/Home/Rehearsal integration"
status: backlog
epic: EPIC-076
parents: ['EPIC-076']
estimate: 5
labels: [m14, calendar, ui]
acceptance_criteria:
  - 'Users can add/edit an exam, attach stacks and individual items (cards, questions, notes), and choose between Custom weeks or StudyFrog proposed weeks (with factor reasoning shown)'
  - 'A schedule preview displays weekly/daily item load and readiness summary before the user applies or re-balances the schedule'
  - 'Applied exam schedule blocks appear in Calendar and Home views and offer a one-click button to start a rehearsal run scoped to that day''s exam items'
  - 'Headless UI and service tests verify exam creation, item attachment, window switching, preview/apply, and accessible widget names'
---

## Context
Users need an intuitive, calm interface to attach materials to an exam, compare StudyFrog's proposed preparation window against their own preference, preview the resulting daily workload, and jump straight into scheduled exam rehearsals.

## Description
Build the Exam Scheduler UI and service wiring in `studyfrog-service` and `studyfrog-ui`:
- Exam configuration dialog/panel with target date, stack/item picker, Custom vs Proposed week selector, and workload preview.
- Integration of scheduled blocks into Calendar (Month/Week/Day), Home ("Coming up" / "Due now"), and one-click scoped Rehearsal launch.
- Headless UI and service integration tests.

## Acceptance Criteria
- [ ] Users can add/edit an exam, attach stacks and individual items (cards, questions, notes), and choose between Custom weeks or StudyFrog proposed weeks (with factor reasoning shown)
- [ ] A schedule preview displays weekly/daily item load and readiness summary before the user applies or re-balances the schedule
- [ ] Applied exam schedule blocks appear in Calendar and Home views and offer a one-click button to start a rehearsal run scoped to that day's exam items
- [ ] Headless UI and service tests verify exam creation, item attachment, window switching, preview/apply, and accessible widget names

## Design Notes
- Follow the `EPIC-070`/`EPIC-073` redesign style (`palette_of(ui)`, `home_hero::card`, `gradient_panel`), Invariant P1, and P6 ($\le 250$ LOC per file).

## Dependencies
- `EPIC-076`, `STORY-0246`, `STORY-0247`
- Child tasks: `TASK-0309`, `TASK-0310`, `TASK-0311`

## Open Questions
None.
