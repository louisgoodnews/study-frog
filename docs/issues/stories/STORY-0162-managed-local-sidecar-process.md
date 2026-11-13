---
id: STORY-0162
type: story
title: "Managed Local Background Sidecar Process and Supervisor"
status: backlog
epic: EPIC-055
parents: ['EPIC-055']
estimate: 5
labels: [m13, agent]
acceptance_criteria:
  - 'SidecarSupervisor spawns local llama-server process on ephemeral loopback port'
  - 'Stdin watchdog pipe and graceful signal termination ensure no orphaned processes'
  - '5-minute idle timer automatically shuts down sidecar process to reclaim 100% of RAM'
---

## Context
Running local LLMs safely without crashing the desktop app or hoarding system RAM.

## Description
Implement a managed local background sidecar supervisor in studyfrog-agent that launches llama-server on demand, enforces loopback security, and auto-shuts down when idle.

## Acceptance Criteria
- [ ] SidecarSupervisor spawns local llama-server process on ephemeral loopback port
- [ ] Stdin watchdog pipe and graceful signal termination ensure no orphaned processes
- [ ] 5-minute idle timer automatically shuts down sidecar process to reclaim 100% of RAM

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-055

## Open Questions
None.
