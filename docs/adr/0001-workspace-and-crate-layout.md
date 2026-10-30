# ADR 0001: Workspace and crate layout

- Status: Accepted
- Date: 2026-10-30

## Context
The constraints require high modularity, small files, and reusable primitives as separate
crates. Compile times and test isolation also benefit from small crates.

## Decision
A single Cargo workspace (resolver 3, edition 2024) with twelve crates under `crates/`:
`core, md, store, srs, rehearsal, slm, report, achieve, board, calendar, ui, app`.
Dependencies are declared once in `[workspace.dependencies]`; members use `workspace = true`.
Lints are shared via `[workspace.lints]`.

Dependency direction: `app → ui → domain crates → core`. Allowed domain edges:
`md→core`, `store→{core,md}`, `srs→core`, `rehearsal→{core,srs}`, `slm→core`,
`report→{core,srs}`, `achieve→core`, `board→core`, `calendar→core`.
No domain crate depends on `ui` or `app`. `app` depends only on `ui`.

> **Amended by [ADR 0006](0006-application-service-layer.md):** a thirteenth crate, `studyfrog-service`,
> sits between `ui` and the domain crates (`app -> ui -> service -> domain -> core`).

## Consequences
+ Domain logic is testable without a GUI; the UI is replaceable.
+ Compile-time enforcement of layering (a forbidden edge needs a manifest change, visible in review).
- More manifests to maintain. Mitigated by workspace inheritance.
- Cross-domain features (e.g. achievements fed by run events) communicate through `core` data
  types, not direct crate calls; the UI/shell composes them.

## Verification
A script check (TASK-0003) parses `cargo metadata` and fails if any forbidden edge appears.

## Alternatives
Single crate with modules: rejected, layering would be convention only.
