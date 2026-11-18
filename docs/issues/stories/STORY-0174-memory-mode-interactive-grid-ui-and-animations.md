---
id: STORY-0174
type: story
title: "Memory Mode Interactive Card Grid UI, Card Back Styling, and Audio-Visual Feedback"
status: backlog
epic: EPIC-059
parents: ['EPIC-059']
estimate: 3
labels: [m14, ui, rehearse]
acceptance_criteria:
  - 'Responsive card grid view dynamically sizing tiles based on window dimensions and pair count'
  - 'Distinguished card-back visual styling featuring StudyFrog frog branding or subtle geometric patterns'
  - 'Clear visual feedback states for hovered cards, face-up text reading, successful match glows, and mismatch shakes/warnings'
---

## Context
A physical memory game depends heavily on visual tactile clarity: cards must look distinct when face-down, flip cleanly to reveal legible text, and visually signal success or error when two cards are flipped.

## Description
Build the interactive user interface for Memory Mode in `crates/studyfrog-ui`:
- Render the tile grid with responsive cell dimensions adapting to 3x4, 4x4, or 4x5 layouts.
- Design face-down card backs using `palette_of(ui)` theme tokens, featuring a branded frog motif or patterned border.
- Render flipped cards with clear typography, wrapping, and subtle elevation/shadows.
- Provide clear color-coded feedback:
  - Accent glow and persistent state for matched pairs.
  - Danger/warning stroke and shake/pause state for mismatches prior to flipping back down.
- Maintain HUD showing elapsed time, moves made, pairs remaining, and current streak.

## Acceptance Criteria
- [ ] Responsive card grid view dynamically sizing tiles based on window dimensions and pair count
- [ ] Distinguished card-back visual styling featuring StudyFrog frog branding or subtle geometric patterns
- [ ] Clear visual feedback states for hovered cards, face-up text reading, successful match glows, and mismatch shakes/warnings

## Design Notes
Use `egui` drawing primitives, strictly adhering to `palette_of(ui)` without hardcoded RGB values. Ensure file line counts remain $\le 250$ LOC per file (Invariant P6).

## Dependencies
EPIC-059, STORY-0173

## Open Questions
None.
