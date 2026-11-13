---
id: STORY-0163
type: story
title: "GGUF Model Catalog, Downloader, and Streaming Generation"
status: backlog
epic: EPIC-055
parents: ['EPIC-055']
estimate: 5
labels: [m13, slm]
acceptance_criteria:
  - 'Catalog supports Qwen2.5-1.5B-Instruct and SmolLM2-360M GGUF models with SHA-256 validation'
  - '1-click model download manager supports HTTP Range resumption and progress reporting'
  - 'Server-Sent Events (SSE) streaming client delivers token-by-token responses to UI'
---

## Context
User-friendly model acquisition and responsive token generation.

## Description
Deliver a vetted GGUF model catalog, resumable HTTP model downloader with checksum verification, and token streaming in chat and exam interfaces.

## Acceptance Criteria
- [ ] Catalog supports Qwen2.5-1.5B-Instruct and SmolLM2-360M GGUF models with SHA-256 validation
- [ ] 1-click model download manager supports HTTP Range resumption and progress reporting
- [ ] Server-Sent Events (SSE) streaming client delivers token-by-token responses to UI

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-055

## Open Questions
None.
