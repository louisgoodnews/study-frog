# Scripting: Python API, runner, events and Console

Status: built in M12 (EPIC-028, STORY-0130 to STORY-0134). Decision: [ADR 0013](../adr/0013-scripting-out-of-process.md).

## 1. Shape

Scripts are child processes. They talk to an instance over the control protocol (STORY-0124): JSON Lines
`{"id", "cmd", "args"}` in, `{"id", "ok", "result"|"error"}` and event lines `{"event", "data"}` out, over a Unix
socket or the standard streams of `studyfrog --control-stdio`. No interpreter is linked into the binary.

## 2. Lifecycle events (`script_events.rs`, `control_events.rs`)

| Kind | Raised when | `data` |
|---|---|---|
| `vault_opened` | the open vault's folder changes to a folder | `root` |
| `vault_closed` | the open vault's folder changes away from a folder | `root` |
| `item_created` | the create dialog's result arrives successfully | `uid`, `title`, `path` |
| `item_saved` | an item page's save succeeds | `uid`, `title`, `path` |
| `rehearsal_finished` | the run on screen reaches `finished` (not `aborted`) | `run`, `mode`, `items`, `events` |
| `script_finished` | a run of the runner ends | `job`, `ok`, `code`, `timed_out`, `error`, `millis`, `stdout`, `stderr` (tails) |

Detection: `from_msg` reads a message before it is applied (create and save results); `from_change` compares a
probe (vault folder, run id and state) before and after. The headless engine checks every applied message; the
window checks the worker's messages and probes once per frame. Events go to an in-process bus (one channel per
subscriber; closed ones are dropped). `subscribe [kinds...]` registers a connection; events are written as
`{"event": "lifecycle", "data": {"kind", "at_ms", "data"}}` until it closes. Unknown kinds are an error. Not raised:
deletions, moves, imports, exams graded.

## 3. Python client (`python/studyfrog/`)

Standard library only; `connect(socket=None, *, home, start, vault, binary, read_only, allow_destructive)`. Socket
order: argument, `$STUDYFROG_SOCKET`, `<app home>/run/control.sock`; otherwise (with `start`) it starts
`studyfrog --control-stdio --no-control-socket`. `Client.call(cmd, *args)` sends one request and returns its result
(event lines met on the way are buffered for `events()`); typed wrappers cover the command table. The binary embeds
the package (`script_client.rs`) and writes it to `<app home>/scripts/lib/studyfrog/` before each run.

## 4. `scripts.toml` (`studyfrog-service/src/scripting/config.rs`)

Top level `python` (default `python3`); `[[job]]` with `name` (unique), `script` (relative, `Normal` path
components only), `args`, `schedule` (cron), `on` (event kinds), `timeout` (1..=3600, default 60),
`access` (`read` default | `write`), `allow_destructive` (bool), `enabled` (default true). Unknown keys, bad
values and duplicate names make that entry an error (`job N: reason`) and it is left out; a file that is not TOML
gives one error and no jobs. The file is read again on every minute and every event, so edits apply without a
restart.

## 5. Cron (`cron.rs`, `due.rs`)

Five fields `minute hour day-of-month month day-of-week`; `*`, `a-b`, lists, `/step` (on `*`, a range, or a start),
month and day names (`jan`..`dec`, `sun`..`sat`), weekday 0 and 7 both Sunday; `@hourly @daily @midnight @weekly
@monthly @yearly @annually`. When both day fields are restricted either may match (classic cron). Matching is
against local time (`utc_to_local` with the offset at the tick). The ticker remembers the last minute checked; a
tick checks every minute after it up to now (at most 60), starts each matching job once, and fires nothing when the
clock went back. Build vs buy: the `cron` crate parses seconds and years and iterates with chrono, which we do not
need; the in-house matcher is about 120 lines and fully tested.

## 6. Runs (`exec.rs`, `script_launch.rs`, `script_runner.rs`, `script_socket.rs`)

- Program: `.py` with `python`, else the script itself; working directory the scripts folder; standard input
  closed; standard output and error read on their own threads, 64 KiB each kept.
- Environment: `STUDYFROG_SOCKET` (the run's socket), `STUDYFROG_HOME`, `STUDYFROG_JOB`, `STUDYFROG_ACCESS`,
  `STUDYFROG_EVENT` (the event JSON, for event-bound runs), `STUDYFROG_BIN`, `PYTHONPATH` (`scripts/lib` first),
  `PYTHONUNBUFFERED=1`.
- Timeout: the child is in its own process group; at the timeout the group is killed. Output still held open by a
  program the script left behind is read for at most 0.5 s more.
- Socket: inside the instance, `run/script-<pid>-<n>.sock` (0600) is served for the run with `Job::RunScoped`, i.e.
  under the job's policy instead of the instance's (`read`: no writes; `write`; destructive only with
  `allow_destructive`). It is removed when the run ends.
- One run per job at a time; a job still running is not started again (an event-bound job is skipped for that
  event). A job is not started by the `script_finished` event of its own run.
- Logging: one `script` entry (info, or error when it failed, timed out or could not start) with the output's last
  2000 characters, and a second entry with standard error when there was any. A `script_finished` event follows.

## 7. Command line (`script_cli.rs`)

- `studyfrog script run FILE|JOB [--access read|write|destructive] [--timeout S] [-- ARGS]`: a job's entry, else a
  file (current folder first, then the scripts folder) with write access. If `run/control.sock` answers, the run
  gets a proxy socket that refuses what the access does not allow and forwards the rest (the instance's own policy
  still applies, so `destructive` also needs the instance started with `--control-allow-destructive`). Otherwise a
  headless instance is started for the run with the access as its policy and `--no-scripts`, and stopped after it.
  The script's output is printed; the exit code is the script's (1 when it timed out or could not start).
- `script list` (exit 1 when the file has errors), `script daemon [--vault PATH]` (= `--control-stdio --daemon`:
  no standard input, serves the socket and runs jobs until the process is ended), `script install-client [DIR]`.
- `--control-stdio` also takes `--no-scripts`.

## 8. Console (`console_ui.rs`, `console_link.rs`, `view/console.rs`)

A screen in the sidebar. A typed line goes through `control_conn::answer` on a thread (served `logs` and
`subscribe` like any connection, other commands into the window's own queue under the window's policy). `script run
JOB|FILE [ARGS]` and `script list` go to the window's runner. All answers and event lines come back through one
channel and become entries (command, answer, error, event, script output); 500 are kept. The Console subscribes to
every lifecycle event when the window starts. It needs the window's control protocol (not with `--no-control`).

## 9. Not covered

Windows (Unix sockets, process groups); calendar, board, diary and settings actions (not in the control protocol);
new card types or importers inside the window's dialogs (compiled-in only, ADR 0008); sandboxing a script's own file
access; catching up jobs missed while no instance ran.
