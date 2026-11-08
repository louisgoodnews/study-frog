---
id: STORY-0083
type: story
title: Google Gemini in models for StudyFrog
status: review
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
- [ ] Users can input a Google Gemini API key, select standard Gemini models (such as gemini-1.5-flash and gemini-1.5-pro), and test connectivity
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

Open: criterion 2 is half met (key and models yes, but there is no explicit "test connectivity" button; a connection is first tried when it is used); never run against the real Google endpoint (the wire follows the provider's documentation, as for the other providers).

## Dependencies
EPIC-028.

## Open Questions
None.
