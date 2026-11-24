---
id: TASK-0296
type: task
title: "Split the UI Msg and Effect enums into per-area sub-enums"
status: done
epic: EPIC-068
parents: ['STORY-0211']
estimate: 2
labels: [ui, tooling, m14]
acceptance_criteria:
  - 'msg.rs, effect.rs, state.rs, exec.rs and lib.rs are each at most 250 lines (scripts/linecount.sh 250)'
  - 'Behaviour is unchanged: every existing studyfrog-ui test passes without changing its expectations'
  - 'No inherent impl blocks; nesting stays below 3 in the changed code'
---

## Context
`Msg` has about 130 variants and `Effect` about 90. Adding the classroom screen (TASK-0293) to the flat
enums pushes `msg.rs`, `effect.rs`, `state.rs`, `exec.rs` and `lib.rs` past the 250-line cap. Each enum
is therefore split by area into sub-enums, with one wrapper variant per area on the top-level enum.

## Description
- `Msg` keeps its wrapper variants and gains one wrapper per area (`Vault`, `Import`, `Browse`, `Editor`,
  `Graph`, `Rehearsal`, `Reports`). Each wrapper holds a sub-enum (`MsgVault`, `MsgImport`, ...) defined in
  `crates/studyfrog-ui/src/msg/<area>.rs` and re-exported from `msg.rs`.
- `Effect` is split the same way into `EffectVault`, `EffectImport`, `EffectBackup`, `EffectBrowse`,
  `EffectEditor`, `EffectRehearsal`, `EffectPlanner`, `EffectAssistant` and `EffectReports`, in
  `crates/studyfrog-ui/src/effect/<area>.rs`.
- Every call site is rewritten to the wrapped form (`Msg::Vault(MsgVault::Started { .. })`). Variant names
  and payloads do not change.
- Nothing is added to the flat enums for the classroom screen; it gets its own sub-enum (TASK-0293).

## Acceptance Criteria
- [x] `msg.rs`, `effect.rs`, `state.rs`, `exec.rs` and `lib.rs` are each at most 250 lines (`scripts/linecount.sh 250`)
- [x] Behaviour is unchanged: every existing `studyfrog-ui` test passes without changing its expectations
- [x] No inherent `impl` blocks; nesting stays below 3 in the changed code

## Design Notes
- Sub-enums live in folder modules next to their parent file (`msg/` next to `msg.rs`, `effect/` next to
  `effect.rs`), so `lib.rs` needs no new module declarations.
- Grouping was done by a one-off rewrite script over the call sites. The rewrite skips string literals so
  help text is unchanged.
- Build vs buy (P10): not applicable, no external crate involved.

## Dependencies
- Blocks `TASK-0293` (classroom setup dialog and lecture chat view), which is ported onto this layout.

## Open Questions
None.
