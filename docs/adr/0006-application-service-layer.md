# ADR 0006: Application service layer

- Status: **Accepted** (2026-10-30, by the owner; implemented in M3)
- Date: 2026-10-30
- Amends: [ADR 0001](0001-workspace-and-crate-layout.md)

## Context
Finishing M3 needs a use case that no existing crate can host: *rate an item*, which reads the item's
stored memory state (`store`), computes the next state (`srs`), and writes a review plus the new state
(`store`), driven by engine output (`rehearsal`). ADR 0001 forbids `store` and `rehearsal` from
depending on each other, and `ui` must stay thin ("renders state and dispatches", design principle on
no business logic in the GUI). The same gap will recur for reindex-on-poll, reports and achievements.

## Decision
Add one crate, `studyfrog-service` (application layer), between the domain crates and `ui`:

```
app -> ui -> service -> { store, srs, rehearsal, ... } -> core
```
- It composes domain crates into use cases (`record_step`, `new_run`, `recover_runs`, ...).
- Pure rules stay in the domain crates; `service` only wires reads, a pure function, and writes
  (for example `apply_rating` is a pure function that merely calls `srs`).
- Allowed dependencies: `core, md, store, srs, rehearsal` for now; it grows as use cases arrive
  (`report`, `achieve`, ...). `ui` may depend on `service`; no domain crate may.
- `scripts/check-deps.sh` enforces the new edges.

## Alternatives
- **Put orchestration in `ui`:** puts business logic next to egui, and its tests would compile the GUI.
- **Let `store` depend on `srs`/`rehearsal`:** turns persistence into the application layer and creates
  a dependency knot between domain crates.
- **Dev-dependency cross-wiring in tests only:** leaves the production use case without a home.

## Consequences
+ One tested place for use cases; `ui` stays a renderer; domain crates stay independent.
- One more crate to maintain; a risk of it becoming a dumping ground. Mitigation: no logic that can
  be a pure function in a domain crate, and a size cap like any other file (250 lines).

## Review request
The owner has not seen this yet. Reverting is cheap (the crate has three small modules).
