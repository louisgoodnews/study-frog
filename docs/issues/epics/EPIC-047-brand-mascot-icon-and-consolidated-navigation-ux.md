---
id: EPIC-047
type: epic
title: Brand mascot icon and consolidated navigation UX
status: done
parents: []
labels: [m12, ui, ux]
acceptance_criteria:
  - "StudyFrog features a distinct, stylized Breviceps macrops (desert rain frog) mascot icon across the window header, about dialog, and brand marks"
  - "Research and prototyping evaluate consolidating secondary navigation items into a floating burger button for distraction-free study"
---

## Context
StudyFrog's brand identity revolves around the studious, serious, and lovable *Breviceps macrops* (desert rain frog)—notable for its round globular body, earnest gaze, and determined posture. Additionally, user feedback suggests exploring cleaner window chrome by consolidating secondary navigation actions into a modern floating burger button.

## Description
Deliver brand and chrome enhancements:
1. **Breviceps Macrops Mascot Icon (`STORY-0146`):** Replace generic geometric circles with a custom vector-drawn *Breviceps macrops* icon (round body, soulful forward-facing eyes, cute studious expression) in `crates/studyfrog-ui/src/view/navbar.rs`.
2. **Floating Navigation Button Research (`STORY-0147`):** Conduct UX research and design a floating burger navigation overlay that frees up window horizontal space while keeping all screens accessible in 1 click.

## Acceptance Criteria
- [x] StudyFrog features a distinct, stylized Breviceps macrops (desert rain frog) mascot icon across the window header, about dialog, and brand marks
- [x] Research and prototyping evaluate consolidating secondary navigation items into a floating burger button for distraction-free study

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/navbar.rs`, `crates/studyfrog-ui/src/view/icons.rs`.
- Child stories: STORY-0146, STORY-0147.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- None.

## Open Questions
- None.
