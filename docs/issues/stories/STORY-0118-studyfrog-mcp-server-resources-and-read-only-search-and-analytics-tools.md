---
id: STORY-0118
type: story
title: StudyFrog MCP server resources and read-only search, analytics, and forecast tools
status: review
epic: EPIC-037
parents: [EPIC-037]
estimate: 5
labels: [m12, agent, report, service]
acceptance_criteria:
  - "The MCP server exposes canonical studyfrog:// resources for vault summary, stacks, items (Markdown + SRS state), reports, and learning diary entries"
  - "Read-only MCP tools (readOnlyHint: true, idempotentHint: true) allow external AI clients to search items via the StudyFrog query language, fetch due queues, read PDF pages, and query retention/forecast/leech reports"
  - "All read-only resources and tools operate with zero side effects on vault files or SQLite schedules"
---

## Context
When a learner connects an external AI client (such as Claude Desktop, Cursor, or Zed) to their StudyFrog vault over MCP, the AI needs structured, read-only visibility into the learner's stacks, items, due queues, PDFs, retention curves, leeches, and forecasts without scraping raw files or guessing YAML schemas.

## Description
Register StudyFrog's canonical read-only MCP Resources and Tier-1 Read-Only Tools in `studyfrog-service`:
1. **`studyfrog://` Resource Templates (`resources/list`, `resources/templates/list`, `resources/read`):**
   - `studyfrog://vault/summary`: JSON summary of vault name, stack tree, total items by kind, due count, and streak.
   - `studyfrog://stacks/{stack}`: JSON metadata, settings, familiarity, and item list for a stack.
   - `studyfrog://items/{id_or_key}`: Full Markdown text (`text/markdown`) and per-stack FSRS/SM-2 state for an item.
   - `studyfrog://reports/{kind}`: JSON serialization of `retention`, `forecast`, `familiarity`, `calibration`, and `runs`.
   - `studyfrog://diary/{date}`: Learning diary entries and self-assessment scale ratings for `YYYY-MM-DD`.
2. **Tier-1 Read-Only Tools (`readOnlyHint: true`, `idempotentHint: true`, `destructiveHint: false`):**
   - `vault_search_items`: Execute a StudyFrog query (`kind:card tag:exam ORDER BY familiarity ASC`) or full-text search and return matching item summaries.
   - `vault_get_due_queue`: Return the prioritized rehearsal queue for a stack or the whole vault.
   - `analytics_get_report`: Return structured learning telemetry (retention, latency, calibration, leeches).
   - `forecast_simulate_workload`: Return the 30-day review/study-minute forecast and backtest metrics.
   - `pdf_read_pages`: Return extracted text and annotations for specified pages of a vault PDF.

## Acceptance Criteria
- [x] The MCP server exposes canonical studyfrog:// resources for vault summary, stacks, items (Markdown + SRS state), reports, and learning diary entries
- [ ] Read-only MCP tools (readOnlyHint: true, idempotentHint: true) allow external AI clients to search items via the StudyFrog query language, fetch due queues, read PDF pages, and query retention/forecast/leech reports
- [x] All read-only resources and tools operate with zero side effects on vault files or SQLite schedules

## Design Notes
- Touchpoints:
  - `crates/studyfrog-service/src/mcp_resources.rs` and `mcp_tools_read.rs`.
- Adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built (in `crates/studyfrog-ui/src/mcp_vault*.rs`, `mcp_search.rs`, `mcp_reports.rs`, `mcp_diary_pdf.rs`, and `crates/studyfrog-service/src/item_schedule.rs`) and tested on sample vaults (`crates/studyfrog-ui/tests/mcp_vault.rs`, `mcp_vault_more.rs`, `mcp_reports.rs`):
- Resources: `studyfrog://vault/summary` (stacks with item, due and new counts, items by kind, due count, reviews today, streak), `studyfrog://stacks/{stack}` (by id, folder with `%2F` for `/`, or title; scheduler, daily limits, familiarity, items), `studyfrog://items/{id}` (by id or item key; the Markdown, then a second content with the FSRS or SM-2 state in every stack: stability and difficulty, or ease, repetitions and interval, with due and last review), `studyfrog://reports/{kind}` for `retention`, `forecast`, `familiarity`, `calibration`, `runs` and `latency`, and `studyfrog://diary/{date}` (entries of a day with study minutes, ratings, text and links).
- Read-only tools (`readOnlyHint: true`, `idempotentHint: true`, `destructiveHint: false`): `vault_search_items`, `vault_get_due_queue` (the queue a spaced run would build now, per stack or vault, within daily limits; builds nothing), `analytics_get_report` (retention, latency, calibration, familiarity, runs, forecast), `forecast_simulate_workload` (expected reviews per day for up to 30 days and the backtest error) and `pdf_read_pages` (text plus highlights and notes of up to 20 pages).
- Zero side effects: a test calls every read tool and resource and compares every Markdown file and every row of every index table (schedules, events, runs, ...) before and after; nothing changes.

Open (depends on EPIC-034, not built yet): leech diagnostics (`analytics_get_report` with `kind: leeches` answers that they are not built), the study-minute forecast (`study_minutes` is `null`), and true retention curves (the retention report is the bucketed recall by gap the Reports screen shows). Criterion 2 stays open until these exist; the rest is done.

## Dependencies
EPIC-037, STORY-0117.

## Open Questions
None.
