---
id: STORY-0060
type: story
title: Settings in tabs
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "Settings are sorted into tabs by domain in a notebook-style tab bar"
  - "The selected tab is remembered, and every tab is reachable by keyboard"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Settings is one long page; group it so a setting can be found by what it is about.

## Acceptance Criteria
- [ ] Settings are sorted into tabs by domain in a notebook-style tab bar
- [ ] The selected tab is remembered, and every tab is reachable by keyboard

## Design Notes
Proposed tabs: General, Appearance, Study and scheduling, Rehearsal, Answers and AI, Notifications, Data (backups, profiles, vault), Subjects and teachers. A tab is a record of a title and a view function, so a task can add one without touching the shell. Keep the existing setting persistence (ADR 0007) as it is; this changes layout only.

Tasks: TASK-0142.

## Dependencies
None.

## Open Questions
Answered by the owner on 2026-11-02: the proposed tabs are accepted.
