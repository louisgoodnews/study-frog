---
id: TASK-0299
type: task
title: "Wizard screens after the vault, reopen from Settings, and wizard commands in the control protocol"
status: backlog
epic: EPIC-069
parents: ['STORY-0212']
estimate: 5
labels: [m14, ui, tooling]
acceptance_criteria:
  - "After the vault is chosen at first boot, the wizard shows the steps; the local-model step reuses the STORY-0206 audit and suggestions"
  - "The remote-provider step is opt-in and states that scored text leaves the machine"
  - "Settings has a button that reruns the wizard"
  - "The control protocol has `wizard status` and `wizard step`"
---

## Context
TASK-0297 and TASK-0298 provide the step model and the downloads; this task puts them on screen.

## Description
Screens for each step, the reopen path in Settings, and the two control commands for agents.

## Acceptance Criteria
- [ ] Wizard screens after the vault, reusing the STORY-0206 panel
- [ ] Remote-provider step is opt-in with a data-leaves-the-machine notice
- [ ] Settings button reruns the wizard
- [ ] `wizard status` and `wizard step` in the control protocol

## Design Notes
Follow the calm UI rules (`docs/specs/ui-calm.md`) and the theme tokens.

## Dependencies
- TASK-0297, TASK-0298.

## Open Questions
None.
