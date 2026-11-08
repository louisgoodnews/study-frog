---
id: STORY-0084
type: story
title: Deepseek in models for StudyFrog
status: review
epic: EPIC-028
parents: [EPIC-028]
estimate: 5
labels: [m12, agent, net]
acceptance_criteria:
  - "The Assistant Settings screen offers Deepseek as a distinct selectable model provider"
  - "Users can configure their Deepseek API key, base URL (defaulting to https://api.deepseek.com), and model selection (deepseek-chat, deepseek-reasoner)"
  - "Interactive exams and prompt actions communicate reliably with Deepseek endpoints and handle rate limits or error responses gracefully"
  - "API keys are stored securely in secrets.toml with 0600 permissions and never logged"
---

## Context
Deepseek provides high-reasoning models via an external API. StudyFrog users want native configuration options for Deepseek to run interactive exams and vault queries without having to manually adapt generic OpenAI base URLs.

## Description
Integrate Deepseek as a first-class model provider:
- Allow users to select Deepseek directly in the Assistant Connections tab.
- Preconfigure the base URL `https://api.deepseek.com` and default models (`deepseek-chat`, `deepseek-reasoner`).
- Authenticate requests using `Authorization: Bearer <key>` over `/chat/completions` (or `/v1/chat/completions`).
- Parse responses from `/choices/0/message/content` and accommodate reasoning tokens if returned by `deepseek-reasoner`.
- Handle API key storage in `secrets.toml` with strict `0600` permissions via `studyfrog-service::agent`.

## Acceptance Criteria
- [x] The Assistant Settings screen offers Deepseek as a distinct selectable model provider
- [x] Users can configure their Deepseek API key, base URL (defaulting to https://api.deepseek.com), and model selection (deepseek-chat, deepseek-reasoner)
- [ ] Interactive exams and prompt actions communicate reliably with Deepseek endpoints and handle rate limits or error responses gracefully
- [x] API keys are stored securely in secrets.toml with 0600 permissions and never logged

## Design Notes
Crate touchpoints:
- `crates/studyfrog-agent/src/provider.rs`: add `Protocol::Deepseek` to `Protocol` enum, update `PROTOCOLS`, `protocol_name(Protocol::Deepseek) => "deepseek"`, and `default_base_url(Protocol::Deepseek) => "https://api.deepseek.com"`.
- `crates/studyfrog-agent/src/wire.rs`: build request using Deepseek URL and authentication header; parse chat reply.
- `crates/studyfrog-ui/src/view/agent.rs`: connection selection button and model presets.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Built and tested against fixed requests and answers (`crates/studyfrog-agent/tests/wire.rs`): `Protocol::DeepSeek` with the default address `https://api.deepseek.com`, `POST /chat/completions` with a Bearer key (required), the reasoning text of `deepseek-reasoner` ignored, and the suggested models `deepseek-chat` and `deepseek-reasoner` as buttons in the connection form. Errors and rate limits (429) use the shared handling, which never echoes the key.

Open: criterion 3 (exams and prompt actions "reliably" against DeepSeek) is not checked; nothing was run against the real DeepSeek endpoint.

## Dependencies
EPIC-028.

## Open Questions
None.
