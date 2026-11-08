# The AI agent

Implements ADR 0009. Nothing is sent anywhere until the user connects a model, and nothing is written to the vault
until the user applies a proposal.

## Connections
`providers.toml` (config folder of the profile) lists connections: `name`, `protocol` (`ollama`, `openai`,
`anthropic`), `base_url`, `model`, `allowed`. API keys are in `secrets.toml` (`[keys]`, mode 0600), never in the
vault or in `providers.toml`. A connection whose host is `localhost`, `127.0.0.1` or `::1` is local and needs no
permission; a remote one is refused until the user ticks "Allow sending text from my vault".
Calls are logged in `logs/agent.log` of the application home: time, connection, host, status. Never text or keys.

## Wire
Ollama `POST /api/chat`, OpenAI-compatible `POST /v1/chat/completions` (Bearer key), Anthropic `POST /v1/messages`
(`x-api-key`, `anthropic-version`), Google Gemini `POST /v1beta/models/{model}:generateContent` (`x-goog-api-key`; the key
is never in the address; a safety stop is reported with its `finishReason`) and DeepSeek `POST /chat/completions`
(Bearer key; reasoning text is not shown). One non-streaming request per turn, 180 s timeout. Errors say what happened
(refused key, wrong model name, unreachable server) and never echo a key.

## Working the vault
The user ticks stacks and writes an instruction. The screen shows how many items and characters would be sent (at
most 30 000 characters, 3 000 per item) and where; a remote model waits for a click on Send, a local one is asked
straight away. The model answers with one JSON object: `summary` and `changes` (`create_note`, `create_card`,
`create_question`, `update_item`). Every change is checked: stacks and ids must be ones it was shown, titles one line,
texts at most 20 000 characters, at most 40 changes. What fails is listed as rejected. The user accepts or rejects each
change and applies them; a failing change does not stop the others. Agent-made items do not get board cards.

## Exams
The model is told the questions and reference answers of the chosen cards and questions (not notes), asks one at a
time and grades each answer. At the end it returns `{"results": [{"id": ..., "verdict": ...}]}`. The screen hides that
JSON and offers "Record as reviews": correct is Good, partly right is Hard, not right is Again, written like a
rehearsal run writes them. Nothing is recorded without that click.
