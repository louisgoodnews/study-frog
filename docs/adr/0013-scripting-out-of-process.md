# ADR 0013: Scripting and the Python API out of process

- Status: **Accepted** (2026-11-08; implemented in M12, EPIC-028)
- Date: 2026-11-08

## Context
EPIC-028 asks for a Python API, scripts run as scheduled jobs, as hooks on lifecycle events (vault opened,
item created, rehearsal finished) and from a console inside the application, and for plugin modding
(importers, custom cards, automated views). The application is one static binary that forbids `unsafe`
(P9) and has no Python linked in. ADR 0008 made extensions compiled-in records of function pointers, which
cannot be installed by a user. The control protocol (STORY-0124: `studyfrog --control-stdio`,
`<app home>/run/control.sock`, `studyfrog ctl`) and the MCP server (ADR 0009) already let another process
read and drive an instance through a policy (read, write, destructive).

## Options
| | embed CPython (`pyo3`) | WASM plugins | child processes over the control protocol |
|---|---|---|---|
| Static binary, no Python linked | no (libpython, ABI per version) | yes, but a heavy runtime crate | yes |
| `unsafe_code = "forbid"` | no (FFI) | yes | yes |
| User writes plain Python, runs right away | yes | no (compile to WASM) | yes |
| A crashing or hanging script hurts the app | yes (same process, GIL) | no | no (own process, timeout) |
| Permissions | whatever the process can do | capability imports | the control policy per script |
| New API surface to keep stable | the Python C API binding | a plugin ABI | the control commands that already exist |

## Decision
Scripts are **child processes** that talk to StudyFrog over the **control protocol**; no interpreter is
embedded.

1. **Python client.** A pure-Python package `studyfrog` (standard library only, `python/studyfrog/` in the
   repository) wraps the control commands in typed functions. It connects to `run/control.sock` (or the
   socket named in `STUDYFROG_SOCKET`) or starts a headless instance (`studyfrog --control-stdio`) and speaks
   JSON Lines over its standard streams. The binary carries the package's source (`include_str!`) and
   writes it to `<app home>/scripts/lib/` so scripts find it without `pip`.
2. **Lifecycle events.** The instance derives events from what it applies (`vault_opened`, `vault_closed`,
   `item_created`, `item_saved`, `rehearsal_finished`, `script_finished`) and publishes them on an in-process
   bus. A connection asks for them with the control command `subscribe [kinds...]`; they arrive as event
   lines `{"event": "lifecycle", "data": {...}}` until the connection closes, like `logs --follow`.
3. **Runner.** `<app home>/scripts/scripts.toml` lists jobs: a script in `<app home>/scripts/`, a cron
   schedule (five fields, local time, parsed in-house) and/or event kinds, a timeout and an access level.
   Every running instance (window or headless, and `studyfrog script daemon`) runs them as child processes
   with a timeout; output and errors go to the application log. `studyfrog script run FILE` runs one now.
4. **Consent.** A script gets a control socket of its own for its run, served with the access its entry
   names: `read` (default), `write`, or `destructive` (`allow_destructive = true`, the per-script form of
   `--control-allow-destructive`). It gets no other channel into the application; what it does on the file
   system is the user's own process rights, so only scripts the user put into their own home run.
5. **Console.** A Console screen sends control commands to its own instance and runs scripts, showing the
   answers, script output and lifecycle events.
6. **Plugins.** The compiled-in registry of ADR 0008 stays for importers and card types that ship with the
   binary. Out-of-process "plugins" are scripts: a script importer reads any file and creates items through
   the API; an automated view is a scheduled or event-bound script that writes a note or an export.

## Consequences
+ No new dependency, no FFI, no packaging of a Python runtime; any language that can open a Unix socket and
  write JSON can script StudyFrog, Python is just the documented one.
+ A script cannot crash or block the application, and a hanging one is killed after its timeout.
+ The API is the control protocol, which is already tested and documented; new commands reach scripts at once.
- A script needs `python3` on the machine and pays a process start (about 30 ms) per run.
- Scripts can only do what the control protocol can do (no calendar, board or settings actions yet), and they
  cannot add a new card type or importer to the window's dialogs.
- The socket is Unix only, so scripting needs Linux or macOS (the stdio transport works anywhere).

## Alternatives
Embedding CPython or a WASM runtime: rejected above. A second JSON-RPC protocol only for scripts: the control
protocol already has the commands, the policy and the transports, so a second one would duplicate them.
