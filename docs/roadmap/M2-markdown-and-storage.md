# M2: Markdown and storage

**Goal:** Vault files parse into core types and are indexed in SQLite.

**Entry criteria:** M1 exit met.

## Epics, stories, tasks

- **EPIC-005** Markdown and front-matter
  - **STORY-0009** Front-matter parse and render
    - TASK-0020 Schema structs and validation
    - TASK-0021 Fence splitter and serde-saphyr parsing
    - TASK-0022 Round-trip property test
  - **STORY-0010** Body parsing
    - TASK-0023 Wikilink extraction
    - TASK-0024 Card and question section split
- **EPIC-006** Storage
  - **STORY-0011** SQLite schema and migrations
    - TASK-0025 Initial schema
    - TASK-0026 Migration runner
  - **STORY-0012** File layer and index sync
    - TASK-0027 Vault scanner
    - TASK-0028 Incremental reindex
    - TASK-0029 File watcher

## Exit criteria

- [ ] A fixture vault of 10k notes scans, parses and indexes under budget
- [ ] `parse(render(x)) == x` property holds for all kinds
- [ ] Renaming a file keeps history (id-based)
- [ ] Corrupt files are reported, not fatal
- [ ] Migrations apply on an empty DB and on the previous version

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
