---
id: STORY-0084
type: story
title: Deepseek in models for StudyFrog
status: done
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
- [x] Interactive exams and prompt actions communicate reliably with Deepseek endpoints and handle rate limits or error responses gracefully
- [x] API keys are stored securely in secrets.toml with 0600 permissions and never logged

## Design Notes
Crate touchpoints:
- `crates/studyfrog-agent/src/provider.rs`: add `Protocol::Deepseek` to `Protocol` enum, update `PROTOCOLS`, `protocol_name(Protocol::Deepseek) => "deepseek"`, and `default_base_url(Protocol::Deepseek) => "https://api.deepseek.com"`.
- `crates/studyfrog-agent/src/wire.rs`: build request using Deepseek URL and authentication header; parse chat reply.
- `crates/studyfrog-ui/src/view/agent.rs`: connection selection button and model presets.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Built and tested against fixed requests and answers (`crates/studyfrog-agent/tests/wire.rs`): `Protocol::DeepSeek` with the default address `https://api.deepseek.com`, `POST /chat/completions` with a Bearer key (required), the reasoning text of `deepseek-reasoner` ignored, and the suggested models `deepseek-chat` and `deepseek-reasoner` as buttons in the connection form. Errors and rate limits (429) use the shared handling, which never echoes the key.

**Test connection (2026-11-08):** every connection on the Connections tab has a **Test connection to <name>** button (`agent_conn_test.rs`, `view/agent_conn_test.rs`). It sends one tiny request with no vault text through the worker's transport and the same `ask` path as every question (so a remote provider needs the user's "Allow sending" first; the button is disabled until then) and shows "Works: ..." or "Connection test failed: <sentence>" under the connection; the key is replaced by `[API key]` even if a server echoes it, and never reaches `agent.log`. Verified by `crates/studyfrog-service/tests/agent_conn_test.rs` (wire, consent, errors, log), `crates/studyfrog-ui/tests/agent_conn_test_ui.rs` (button, outcome, disabled until allowed, worker, a 429 during an exam or a prompt ends the wait with the sentence) and `crates/studyfrog-app/tests/providers_fake_server.rs`: the application's real HTTP sender (`studyfrog_net::send`) against a fake server on 127.0.0.1 that answers in the provider's documented format.
Over the real sender: the test, an exam turn and a prompt action answered (`POST /chat/completions`, `Bearer` key, `reasoning_content` ignored), then two 429 answers (exam and prompt) and a 503 that end as sentences ("too many requests; wait a moment") without the key, and a provider that is not running.

Limitation: verified offline only; nothing was sent to the real DeepSeek endpoint (no key or network access here). Criterion 3 is ticked as far as offline checks can show: the exam and prompt paths work end to end over HTTP in DeepSeek's format and errors and rate limits end gracefully.

## Dependencies
EPIC-028.

## Open Questions
None.
