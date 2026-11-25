---
id: STORY-0212
type: story
title: "First-run wizard: optional model setup after the vault, with confirmed downloads"
status: backlog
epic: EPIC-069
parents: ['EPIC-069']
estimate: 5
labels: [m14, slm, ui, tooling]
acceptance_criteria:
  - "After the vault is chosen at first boot, the wizard offers the optional steps local model, remote provider, MiniLM and summary; each can be skipped"
  - "Each download (MiniLM, suggested Ollama model) shows size and source and needs a click; checksums are verified; a failed download can be retried"
  - "The wizard can be reopened from Settings and from the control protocol (`wizard status`, `wizard step`)"
---

## Context
See EPIC-069.

## Description
A pure step model (TASK-0297), confirmed downloads with checksums (TASK-0298), and the screens with Settings and control
protocol hooks (TASK-0299).

## Acceptance Criteria
- [ ] Optional steps after the vault, each skippable
- [ ] Confirmed, checksum-verified downloads with retry
- [ ] Reopen from Settings and from the control protocol

## Design Notes
Step order: local model, remote provider, MiniLM, summary. Remote providers are opt-in.

## Dependencies
- EPIC-069; STORY-0206, STORY-0083, STORY-0084.

## Open Questions
- Resolved 2026-11-25: STORY-0206 now allows any model provider with the source shown and confirmed.
