# Storage schema

Status: implemented in `studyfrog-store` (`schema.rs`, migration v1). See [ADR 0003](../adr/0003-persistence-strategy.md).

## Principles
- **Derived index** (rebuildable from the vault): `file`, `item`, `link`, `tag`. Dropping them loses nothing.
- **Authoritative history**: `review`, `mem_state`, `sm2_state`, `run`, `run_queue`, `run_event`. They
  reference items by ULID text but have **no foreign keys**, so deleting or renaming a file never
  deletes history.
- Pragmas: `journal_mode = WAL`, `foreign_keys = ON`, `synchronous = NORMAL`.
- Versioning: `PRAGMA user_version`; one transaction per migration; a database newer than the app is refused.
- Uids are the 26-character ULID text from front-matter.

## Tables (v1)
| Table | Purpose | Key |
|---|---|---|
| `file` | every `.md` in the vault: size, mtime (ns), content hash, uid or parse error | `path` (vault-relative, `/`-separated) |
| `item` | one parsed item per good file: kind, title, owning stack folder, timestamps | `uid`, unique `path` (cascades from `file`) |
| `link` | wikilinks per item | (`src_uid`, `seq`) |
| `tag` | tags per item | (`uid`, `tag`) |
| `review` | one row per recorded review | `id` |
| `mem_state` | FSRS state and due time per item | `item_uid` |
| `sm2_state` | SM-2 state and due time per item | `item_uid` |
| `run`, `run_queue`, `run_event` | rehearsal runs and their append-only event log | see `schema.rs` |

## Run tables (schema v2)
`run` stores the plan next to the run: `order_name`, `effect`, `seed`, `run_limit`, `requeue_again` (the item
filter is not stored; the resulting queue is, in `run_queue`). `run_event` rows are `(run_uid, seq)`-unique and
append-only; `payload` holds the answer text or the rating (1-4). On start-up `abort_unfinished_runs`
turns runs left idle, running or paused into `aborted` and closes their log with a `run_aborted` event.
`review` and `mem_state` are written by the service layer only for `update_schedule` runs.

## Vault rules (M2)
- Only `.md` files are indexed; names starting with `.` (and everything inside such folders) are skipped, as are symlinks.
- A **stack** is a folder containing `_stack.md`. An item belongs to the nearest ancestor folder that is a
  stack; `_stack.md` belongs to its own folder. Items outside any stack (or whose `_stack.md` failed to
  parse) go to the implicit **Inbox** stack (uid 0) when a library is built.
- A file that fails to parse is recorded in `file.error` and has no `item` row; it never aborts a scan.
- A uid used by more than one file marks **every** such file as an error (neither is indexed).
- Files with an error are re-read on every reindex, so fixing or deleting one of two duplicates heals the other.
- Unchanged files (same size and mtime) are not read at all; touched-but-identical files (same content
  hash) only refresh their stat data.
- A rename keeps its uid, so `item` is re-pointed to the new path and history is untouched.
- **Change detection** is polling: `poll_changes` stats the vault, compares a fingerprint, and reindexes
  only on change. Files modified within the settle window (default chosen by the caller, 200 ms
  suggested) are deferred to the next poll. The clock is a parameter.
