---
id: STORY-0169
type: story
title: "Contextual Tooltips, Hover Hints, and Keyboard Shortcut Badges"
status: backlog
epic: EPIC-057
parents: ['EPIC-057']
estimate: 3
labels: [m14, ui, ux]
acceptance_criteria:
  - 'Every icon button on the navigation rail, header bars, and action trays exhibits an explanatory tooltip on hover'
  - 'Tooltips display associated keyboard shortcuts where available (e.g. F1, Space, 1-4, Ctrl+F)'
  - 'Tooltip delays and styling conform to theme token contrast rules and never obscure clicked controls'
---

## Context
Navigation feels difficult and unintuitive because action icons and buttons lack hover explanations or keyboard shortcut cues.

## Description
Conduct a systematic audit and implementation of hover tooltips across the UI:
- Add `on_hover_text` or rich `on_hover_ui` badges to all navigation rail icons (Home, Stacks, Calendar, Rehearsal, Exam, Graph, Scripts, Settings).
- Include hotkey badges in tooltips for rehearsal grading buttons (1: Again, 2: Hard, 3: Good, 4: Easy), flip card (Space), and search (`/`).
- Provide brief 1-line explanations for complex buttons so users immediately understand their purpose before clicking.

## Acceptance Criteria
- [ ] Every icon button on the navigation rail, header bars, and action trays exhibits an explanatory tooltip on hover
- [ ] Tooltips display associated keyboard shortcuts where available (e.g. F1, Space, 1-4, Ctrl+F)
- [ ] Tooltips conform to theme token contrast rules and never obscure clicked controls

## Design Notes
Implement consistent helper functions in `studyfrog-ui::view::widget` for formatted tooltips with shortcut badges.

## Dependencies
EPIC-057

## Open Questions
None.
