---
id: STORY-0124
type: story
title: Interactive CLI control protocol and console log streaming for AI agents
status: review
epic: EPIC-038
parents: [EPIC-038]
estimate: 8
labels: [m12, service, ui, app]
acceptance_criteria:
  - "AI agents can operate StudyFrog interactively via studyfrog-app ctl <command> or a persistent JSON-RPC/JSONL control stream (--control-stdio or ~/.local/share/studyfrog/run/control.sock)"
  - "The control protocol provides full coverage of application features: screen navigation, UI widget/state inspection, item/stack CRUD, rehearsal and exam run stepping, query execution, imports/exports, and screenshots"
  - "Agents can read and live-stream structured console logs, notices, and error traces via the control protocol"
---

## Context
To let AI agents interactively operate StudyFrog with full access to every application feature and console log (`EPIC-038`), StudyFrog needs a deterministic, scriptable CLI control protocol. Because StudyFrog's UI is built on a pure Elm-style architecture (`State` + `Msg` -> `(State, Vec<Effect>)`), the control protocol can inspect the live `State`, dispatch any UI `Msg` or `studyfrog-service` action, wait for background `Effect` completion, and stream console logs from `~/.local/share/studyfrog/logs/console.jsonl` (`STORY-0122`).

## Description
Implement the interactive CLI Control Protocol across `studyfrog-ui`, `studyfrog-service`, and `studyfrog-app`:
1. **Control Transport Modes (`crates/studyfrog-app`):**
   - **Live Socket Attach (`studyfrog-app ctl ...`):** Connects to a running desktop or headless StudyFrog instance via `~/.local/share/studyfrog/run/control.sock` to inspect or drive the live window in real time.
   - **Headless Interactive Stream (`studyfrog-app --control-stdio [--vault <path>]`):** Runs StudyFrog headlessly, reading JSON control commands from `stdin` and emitting structured JSON state/event/log responses on `stdout`.
2. **Full Application Feature Coverage:**
   - `state`: Return a structured JSON snapshot of the active screen, open vault, selected stack/item, active rehearsal/exam prompt, modal state, and recent notices.
   - `nav <screen>`: Navigate to any screen (`home`, `folders`, `items`, `rehearse`, `calendar`, `board`, `reports`, `diary`, `achievements`, `graph`, `assistant`, `settings`, `help`) or `back` / `forward`.
   - `item <create|open|edit|delete|color|move>` and `stack <create|open|move|settings>`: Full item and stack operations.
   - `rehearse <setup|start|flip|answer|rate|mark|skip|pause|stop>`: Interactively step through rehearsal and exam runs.
   - `query <expr>`: Execute any StudyFrog query or search and return matching items.
   - `screenshot <path.png>`: Render the current frame to a PNG file.
   - `logs [--tail N] [--follow] [--level info|warn|error]`: Return or stream structured console log entries (`console.jsonl`).

## Acceptance Criteria
- [x] AI agents can operate StudyFrog interactively via studyfrog-app ctl <command> or a persistent JSON-RPC/JSONL control stream (--control-stdio or ~/.local/share/studyfrog/run/control.sock)
- [ ] The control protocol provides full coverage of application features: screen navigation, UI widget/state inspection, item/stack CRUD, rehearsal and exam run stepping, query execution, imports/exports, and screenshots
- [x] Agents can read and live-stream structured console logs, notices, and error traces via the control protocol

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/control.rs`: pure command parser and `State` -> JSON summary serializer.
  - `crates/studyfrog-app/src/control_server.rs`: Unix socket and `--control-stdio` loop.
- Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built (the binary is `studyfrog`, not `studyfrog-app`):
- **Transports.** `studyfrog --control-stdio [--vault PATH] [--control-read-only] [--control-allow-destructive] [--no-control-socket]` runs headless (the `Model` and a worker in one thread, every command settled before it is answered): one request per line on standard input, one JSON answer per line on standard output, diagnostics on standard error only. It also serves `<app home>/run/control.sock`. The window serves the same socket (unless `--no-control`; not for `--smoke-frames`/`--screenshot`): requests are taken in the frame loop, one phase per idle frame, answered once the worker has nothing left (an effect counter in `runtime.rs`). `studyfrog ctl [--socket PATH] <command> [args...]` sends one request and prints the answer (exit 0 on success, 1 on an error answer, 2 when no instance answers).
- **`run/` files.** `run/` is created mode 0700 and the socket 0600; `run/control.json` names the pid, socket, mode (window or headless), vault, start time, version and protocol. A socket that answers belongs to another instance and is left alone (that instance is reported on standard error); a stale one (nothing answers) is replaced; both files are removed on a normal exit. A killed window leaves them until the next start replaces them. A socket path longer than the Unix limit (about 107 bytes) cannot be bound and is reported.
- **Requests.** A line of words (`item edit FLASHCARD_1 back "la réponse"`) or JSON `{"id": any, "cmd": "nav", "args": ["home"]}`; answers `{"id", "ok": true, "result"}` or `{"id", "ok": false, "error"}`; log events `{"event": "log", "data": {...}}`. A command whose work raises an error notice, or a create dialog that refuses, answers with an error. `help` returns the command table below.
- **Policy** (like `McpPolicy`): reading and navigating always; writes (create, edit, move, rehearse, import, export, open/close vault, screenshot) unless `--control-read-only`; destructive commands (`item delete`) only with `--control-allow-destructive`.
- **Logs.** `logs [--tail N] [--level info|warn|error]` returns entries of `logs/console.jsonl`; `--follow` keeps streaming new entries as events on the same connection until it closes. Notices are logged in headless mode too, and every refused or failed request is logged as a `warn` entry with source `control` (the error trace of the protocol).

Command table (pure parser `control_parse`/`control_words`, plans of messages `control_plan`, snapshot `control_state`/`control_parts`):

| Command | Becomes | Class |
|---|---|---|
| `state` | snapshot: screen, vault and stacks, selection, item page (fields, dirty, conflict), dialog, rehearse setup and run (phase, prompt, answer once revealed, score), library rows, search results, notice, toasts, last 10 notifications, history | read |
| `nav <screen>`, `back`, `forward` | `Msg::Navigate`, `ChromeMsg::History(Back/Forward)` | read |
| `vault open <dir>`, `vault close` | `Msg::OpenRecent`, `Msg::CloseVault` | write |
| `item create <note/card/question> <title> [--text] [--answer] [--tags] [--stack]` | `ModalMsg::OpenCreate, SetKind, SetParent, Edit, Submit` -> `Effect::CreateItem` | write |
| `item open <ULID/KEY>` | `Msg::OpenItem` -> `Effect::LoadDocument` | read |
| `item edit <item> <title/tags/front/back/body/prompt/reference/fieldN> <text>`, `item color <item> <colour/none>` | `OpenItem`; then `EditAction::Title/Tags/Field/Color`, `SaveClicked` -> `Effect::SaveDocument` | write |
| `item move <item> <stack>` | `BulkMsg::MoveOne(MoveTo)` (undoable) | write |
| `item delete <item>` | `OpenItem`; then `EditAction::AskDelete, ConfirmDelete` | destructive |
| `stack create <title> [--parent]` | the create dialog with `NewKind::Stack` | write |
| `stack open <folder>`, `stack settings <folder>` | `Navigate(Library), Select(Stack)`; settings also answer scheduler and daily limits | read |
| `stack move <folder> [<into>]` | `StackViewMsg::MoveStack` | write |
| `rehearse setup [--preset] [--mode] [--limit] [--stack]...` | `Navigate(Rehearse)`, `SetupEdit::Preset/Mode/Limit/Stack` -> `Effect::PreviewQueue` | read |
| `rehearse start / flip / skip / pause / resume / stop / leave` | `StartRunClicked`; `RunAction(Flip/Skip/Pause/Resume/Abort)`; `LeaveRun` | write |
| `rehearse answer <text>`, `rate <again..easy>`, `mark <easy/medium/difficult>` | `AnswerEdited` + `RunAction(SubmitAnswer)`; `RunAction(Rate)`; `RunAction(Mark)` | write |
| `query <expr>` | `Navigate(Library), Select(Nothing), SearchMsg::Mode(query), Query(expr)`; answers the matching rows (at most 500) or the query error | read |
| `search <text>` | `ChromeMsg::SearchEdited` -> `Effect::SearchItems` and the PDF search | read |
| `export <json/toml/ron/csv/markdown> <dir> [--stack] [--scheduling]` | `TransferMsg::Open(Export), Format, Destination, Mode, Export` | write |
| `import <file> [--stack]` | `TransferMsg::Open(Import), Path, Preview`; then `Import` | write |
| `screenshot <file.png>` | `ViewportCommand::Screenshot` (window only; headless answers that it has no window) | write |
| `logs [--tail] [--follow] [--level]` | served by the connection from `console.jsonl` | read |
| `notices`, `dismiss`, `close`, `settle`, `help` | notifications; `DismissNotice`; `ModalMsg::Close, Discard`; wait; the table | read |

Verified: unit tests of the parser, policy, plans, answers and snapshot (`tests/control_parse.rs`, `tests/control_state.rs`); whole headless sessions over in-memory streams against a real vault and database (navigation, query, create, edit, colour, stack create, move, a rehearsal with a saved review, export and import, policy, logs; `tests/control_session.rs`); the window's frame-by-frame pump (`tests/control_window.rs`); a real Unix socket in a temp dir with the `run/` files, permissions, a refused second instance, a replaced stale socket, a client session, `ctl`'s exchange and a followed log (`tests/control_socket.rs`); the real binary over pipes and `studyfrog ctl` attached to it (`crates/studyfrog-app/tests/control_binary.rs`). Tried by hand: the real window driven with `studyfrog ctl` (state, stack open, rehearse setup/start/flip, two screenshots written and looked at).

Exam runs (added with STORY-0129, 2026-11-08): `rehearse setup --mode exam`, `rehearse type <text>` (the draft), `rehearse submit` (submit and lock it), `rehearse answer <text>` (both), `rehearse finish` (leave the result; refused while the exam runs or when there is no exam), `rehearse stop` (not graded). In an exam `flip`, `skip`, `rate`, `mark` and a blank answer answer with an error. `state` carries `rehearse.run.exam` (items, answered, the locked answers with their scores, `grading`, `graded`, `stopped`, `result` with percent, grade, German grade, passed). Verified headless over in-memory streams against a real vault (`tests/control_exam.rs`: a whole exam graded and stored with no review written, the refusals, a stopped exam left ungraded), with the real binary over pipes (`--control-stdio`, an exam of four items graded B / 2.0) and with `studyfrog ctl` attached to a headless instance's socket (an exam answered to an A+ / 1+, `ctl rehearse flip` refused with exit 1).

Open (why criterion 2 is still not ticked): no AccessKit or widget tree, only the semantic snapshot; calendar, board, diary, reports, achievements, settings, assistant, PDF and folder operations are reachable by `nav` and visible only as the screen name (their contents and actions are not in the protocol); no stack delete, stack settings are read-only; screenshots only from a window (headless has no renderer); no arbitrary `Msg` passthrough (messages are not serialisable). The answer comparison headless uses the host's runtime as the window does.

## Dependencies
EPIC-038, STORY-0122.

## Open Questions
None.
