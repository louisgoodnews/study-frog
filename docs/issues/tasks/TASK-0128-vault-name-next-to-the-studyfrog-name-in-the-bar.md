---
id: TASK-0128
type: task
title: Vault name next to the StudyFrog name in the bar
status: ready
epic: EPIC-023
parents: [STORY-0056]
estimate: 2
labels: [m10, ui]
acceptance_criteria:
  - "The bar shows the vault name next to the StudyFrog name"
  - "Long names are shortened; the tooltip carries the full name and path"
  - "A headless view test and the accessibility audit cover it, and UX_EVIDENCE.md records the reversal of STORY-0041"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add a vault label to the bar in `view/bars.rs`, fed from the open vault in state.

## Acceptance Criteria
- [ ] The bar shows the vault name next to the StudyFrog name
- [ ] Long names are shortened; the tooltip carries the full name and path
- [ ] A headless view test and the accessibility audit cover it, and UX_EVIDENCE.md records the reversal of STORY-0041

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
