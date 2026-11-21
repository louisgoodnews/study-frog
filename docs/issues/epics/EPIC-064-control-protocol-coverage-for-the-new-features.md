---
id: EPIC-064
type: epic
title: "CLI control protocol coverage for the new features"
status: in_progress
parents: []
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'Every feature shipped in the v1.0 milestones can be driven through the control protocol, with a help row and a test per command family'
  - 'Each new feature family has its own story under this epic, and the epic is continuous: stories are added as features land'
  - 'Read commands answer with the data they read; write commands are refused with the reason when the app refuses them'
---

## Context
EPIC-038 built the control protocol for the features of its time. The features added since (homework, the import destinations and PDF cards, search focus, tooltips, appearance and the calendar's homework markers) cannot yet be driven from the command line, so an agent or a script cannot test or use them. This epic is continuous: each new feature family gets a story here as it lands.

## Description
- Add command families to `control_words`, `control_plan`, `control_reply`, `control_policy` and `control_help`, in submodules of those files (the `lib.rs` module list is at its size limit).
- Keep the protocol's rules: reads answer with data, writes are classed in the policy and refused with the reason when the app refuses them, and every command has a `help` row.
- Test each family the way `tests/control_homework.rs` does: words, class, plan and answer.

## Acceptance Criteria
- [ ] Every feature shipped in the v1.0 milestones can be driven through the control protocol, with a help row and a test per command family
- [ ] Each new feature family has its own story under this epic, and the epic is continuous: stories are added as features land
- [ ] Read commands answer with the data they read; write commands are refused with the reason when the app refuses them

## Design Notes
A command family is done when its commands parse, plan, answer and appear in `help`, and its tests pass. Coverage is measured against the stories of the other epics, not against the code.

## Dependencies
Extends EPIC-038 (the protocol itself).

## Open Questions
None.
