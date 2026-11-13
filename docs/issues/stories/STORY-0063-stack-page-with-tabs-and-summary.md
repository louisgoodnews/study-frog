---
id: STORY-0063
type: story
title: "Stack page with tabs and summary"
status: done
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, ui]
acceptance_criteria:
  - "A stack opens as a page with a header and tabs, in the manner of Jira's project view, and the last tab is remembered per stack"
  - "The Summary tab shows what happened in the last 7 days, a status overview, priorities, types and recent activity"
  - "The stack's own details open from the header in the existing stack editing page, which is the issue-like detail view and stays as it is, so the overview and the detail view are two screens"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
The stack page puts everything in one scrolling column. Give it a header and tabs, and a Summary like the one in the owner's Jira screenshots.

## Acceptance Criteria
- [x] A stack opens as a page with a header and tabs, in the manner of Jira's project view, and the last tab is remembered per stack
- [x] The Summary tab shows what happened in the last 7 days, a status overview, priorities, types and recent activity
- [x] The stack's own details open from the header in the existing stack editing page, which is the issue-like detail view and stays as it is, so the overview and the detail view are two screens

## Design Notes
Header: breadcrumb of parent stacks, the stack's name, status chip and actions. Tab bar with an overflow menu ('More') for the tabs that do not fit. Summary mirrors the Jira summary: four count cards (completed, updated, created and due soon, over the last or next 7 days), a status overview as a ring chart with the total in the middle and a legend, a priority bar chart, a breakdown by item type (cards, notes, questions) and an activity feed (empty state: 'No activity yet'). The tabs are Summary and the sub-views of STORY-0064. Charts follow the dataviz rules: colour is never the only cue and values are in the legend. The tab widget is the one from STORY-0060. Sub-views of the tabs are STORY-0064. This page is the stack overview, Jira's project view. The stack's own details (description, fields, priority, subjects, scheduler) are the existing item page, which already serves a note, a card, a question and a stack like Jira's issue view and which the owner accepted as it is on 2026-11-02; it is reached from the header, so there is no Details tab here.

Tasks: TASK-0149, TASK-0150, TASK-0151.

## Dependencies
STORY-0060 (tab widget), STORY-0061 (statuses) and STORY-0058 (familiarity) for the content.

## Open Questions
Answered by the owner on 2026-11-02: the 'stack item view' is the stack's own detail page (Jira's issue view), and that is the stack editing page we have, which stays as it is; this overview (Jira's project view) is the new part. Sub-stacks appear both as a section of Summary and as a Sub-stacks tab (the owner's answer, 2026-11-02): the section shows the first level with status and progress, the tab (STORY-0064) the whole tree
