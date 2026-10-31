# ADR 0009: An AI agent that works the vault and examines the user

- Status: **Accepted** (2026-10-30, by the owner; implemented in STORY-0049, see docs/specs/agent.md)
- Date: 2026-10-30

## Context
The owner wants a local (Ollama and similar) or remote (OpenAI, Anthropic and similar) model to work the vault
like an LLM wiki (the agent keeps linked Markdown pages and an index of what it knows from the user's sources)
and to run interactive exam sessions in the app. A remote provider receives text from the vault, so this is
the first feature that sends user content off the machine.

## Decision
1. **Providers** are plain data: a name, a protocol (`ollama`, `openai-compatible`, `anthropic`), a base URL and
   a model name. One function per protocol turns a conversation into a request and a reply into text. No SDK;
   a small HTTP client crate is the only new dependency (P10: HTTP and TLS are a hard general problem).
2. **Keys and files.** StudyFrog creates its own directories on first use: configuration in
   `~/.config/studyfrog/`, data and logs in `~/.local/share/studyfrog/` and `~/.local/state/studyfrog/`
   (XDG). API keys live in `~/.config/studyfrog/secrets.toml` with mode `0600`, never in the vault, never in
   backups unless the user ticks that, never in logs. The settings screen shows that a key is set, not the key.
3. **The agent proposes; the user applies.** It may read items and propose changes (new or edited items,
   links, an index page). Nothing is written until the user clicks Apply on the proposal. Proposals are shown
   as diffs.
4. **Consent.** A remote provider is off until the user turns it on and sees, in words, which text is sent.
   The agent only reads the stacks the user selects for that session. Local providers need no consent screen.
5. **Exam sessions** are a conversation: the agent asks, the user answers, the agent grades with the same
   verdicts as the answer comparison (correct, partly, not quite) and the result is recorded as a normal
   review only if the user accepts it.
6. **Failure.** Every provider call has a timeout, errors are shown in words, and nothing is retried silently.

## Consequences
+ A local model keeps every byte on the machine; a remote one sends only what the user allowed.
+ The vault stays plain Markdown the user can read and version without the agent.
- A network dependency and a secrets file are new things to secure and test.
- Model output is untrusted text: proposals are validated like imports before they can be applied.
