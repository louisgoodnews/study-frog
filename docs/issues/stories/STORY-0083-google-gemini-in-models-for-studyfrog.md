---
id: STORY-0083
type: story
title: "Google Gemini in models for StudyFrog"
status: done
epic: EPIC-028
parents: [EPIC-028]
estimate: 5
labels: [m12, agent, net]
acceptance_criteria:
  - "The Assistant Settings screen offers Google Gemini as an API provider option alongside Ollama, OpenAI, and Anthropic"
  - "Users can input a Google Gemini API key, select standard Gemini models (such as gemini-1.5-flash and gemini-1.5-pro), and test connectivity"
  - "Requests are transmitted to the Google Generative Language API endpoint (v1beta/models/{model}:generateContent) with x-goog-api-key authentication and correct wire schemas"
  - "Responses and error reasons (including safety finishReason codes) are parsed without data leaks, with API keys preserved in secrets.toml"
---

## Context
Users want to connect Google Gemini via API to assist with vault maintenance, prompt execution, and interactive rehearsal examinations in StudyFrog. Currently, `studyfrog-agent` supports Ollama, OpenAI-compatible, and Anthropic endpoints.

## Description
Implement Google Gemini API support within `studyfrog-agent` and `studyfrog-net`:
- Add `Protocol::Gemini` to `provider.rs`.
- Implement wire serialization and deserialization for Google's `generateContent` API endpoint (`https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`).
- Pass the API key using the `x-goog-api-key` header, preventing key leakage into URL logs.
- Format requests using Gemini's wire schema: map system instructions to `"system_instruction": {"parts": [{"text": system}]}` and dialogue turns to `"contents": [{"role": "user"|"model", "parts": [{"text": ...}]}]`.
- Parse responses from `/candidates/0/content/parts/0/text` and handle `finishReason` safety filters.
- Store keys securely in `secrets.toml` with `0600` permissions via `studyfrog-service::agent`.

## Acceptance Criteria
- [x] The Assistant Settings screen offers Google Gemini as an API provider option alongside Ollama, OpenAI, and Anthropic
- [x] Users can input a Google Gemini API key, select standard Gemini models (such as gemini-1.5-flash and gemini-1.5-pro), and test connectivity
- [x] Requests are transmitted to the Google Generative Language API endpoint (v1beta/models/{model}:generateContent) with x-goog-api-key authentication and correct wire schemas
- [x] Responses and error reasons (including safety finishReason codes) are parsed without data leaks, with API keys preserved in secrets.toml

## Design Notes
Crate touchpoints:
- `crates/studyfrog-agent/src/provider.rs`: add `Protocol::Gemini` to `Protocol` enum, update `PROTOCOLS`, `protocol_name(Protocol::Gemini) => "gemini"`, and `default_base_url(Protocol::Gemini) => "https://generativelanguage.googleapis.com"`.
- `crates/studyfrog-agent/src/wire.rs`: add `format_gemini_request` and `parse_gemini_response`. The model parameter is embedded in the endpoint path (`/v1beta/models/{model}:generateContent`). Roles map `Role::User => "user"` and `Role::Assistant => "model"`.
- `crates/studyfrog-ui/src/view/agent.rs` and `crates/studyfrog-service/src/agent.rs`: UI connection form updates and secret handling.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Built and tested against fixed requests and answers (`crates/studyfrog-agent/tests/wire.rs`): `Protocol::Gemini`, the `generateContent` request with the key in `x-goog-api-key` only, `system_instruction` and `contents` with the `model` role, answers joined from all text parts, safety and block reasons reported without the text, and suggested models as buttons in the connection form.

**Test connection (2026-11-08):** every connection on the Connections tab has a **Test connection to <name>** button (`agent_conn_test.rs`, `view/agent_conn_test.rs`). It sends one tiny request with no vault text through the worker's transport and the same `ask` path as every question (so a remote provider needs the user's "Allow sending" first; the button is disabled until then) and shows "Works: ..." or "Connection test failed: <sentence>" under the connection; the key is replaced by `[API key]` even if a server echoes it, and never reaches `agent.log`. Verified by `crates/studyfrog-service/tests/agent_conn_test.rs` (wire, consent, errors, log), `crates/studyfrog-ui/tests/agent_conn_test_ui.rs` (button, outcome, disabled until allowed, worker, a 429 during an exam or a prompt ends the wait with the sentence) and `crates/studyfrog-app/tests/providers_fake_server.rs`: the application's real HTTP sender (`studyfrog_net::send`) against a fake server on 127.0.0.1 that answers in the provider's documented format.
Over the real sender: the test, an exam question and a 400 answer that echoes the key; the request goes to `POST /v1beta/models/gemini-1.5-flash:generateContent` with `x-goog-api-key` and no key in the URL.

Limitation: verified offline only; nothing was sent to the real Google endpoint (no key or network access here). The wire follows Google's documentation, as for the other providers.

## Dependencies
EPIC-028.

## Open Questions
None.
