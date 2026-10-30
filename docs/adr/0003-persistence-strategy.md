# ADR 0003: Persistence strategy

- Status: **Accepted** (2026-10-30)
- Date: 2026-10-30

## Context
Content must be plain Markdown the user owns. Review history, run events, board and calendar
are structured, queryable, and write-heavy.

## Decision
**Storage split.**
| Data | Home | Reason |
|---|---|---|
| Notes, cards, questions, stack definitions | Markdown files in the vault | user-owned, diffable, tool-agnostic |
| Link graph, search index, item lookup | SQLite (derived, rebuildable) | fast queries; can be dropped and rescanned |
| Reviews, run events, latency, achievements, board, calendar | SQLite (authoritative) | structured, transactional, aggregatable |

**Driver:** `rusqlite` (bundled SQLite, synchronous). No async runtime is needed: the shell
runs store effects on a worker thread. `sqlx` is rejected for compile-time cost and async
overhead with no benefit for a single-user local file.

Item identity: every file has a stable `id` (ULID) in front-matter; SQLite rows reference the
id, so renaming or moving files does not lose history.

Schema changes use ordered SQL migrations tracked with `PRAGMA user_version`. WAL mode on.

## Consequences
+ Deleting the DB loses only history, never content.
- Two stores can disagree; the scanner reconciles by id and content hash (STORY-0012).
- Only `studyfrog-store` touches the filesystem and SQLite.
