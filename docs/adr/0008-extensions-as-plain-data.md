# ADR 0008: Extensions as plain data

- Status: **Accepted** (2026-10-30, by the owner; implemented in M8)
- Date: 2026-10-30

## Context
M8 asks for documented extension points for card types and importers, and an example extension built without
modifying the core crates. The workspace forbids `unsafe` and inherent `impl` blocks (ADR 0001, principle P1)
and prefers plain data with free functions.

## Options
| | trait objects (`Box<dyn Importer>`) | records of function pointers | dynamic loading (`libloading`, WASM) |
|---|---|---|---|
| Fits "no OOP, data first" | partly (behaviour behind objects) | yes: a value you can list and compare | no |
| `unsafe_code = "forbid"` | yes | yes | no for native libraries; a WASM runtime is a heavy dependency |
| Extension needs state | yes | no (a `fn` cannot capture) | yes |
| Testable as data | no | yes | no |
| Deployable without recompiling | no | no | yes |

## Decision
Records of function pointers (`Importer`, `CardType`) in a dependency-free crate `studyfrog-ext`, held in a
`Registry` that is two `Vec`s. An extension is a crate with `register(&mut Registry)`; the application binary
calls it, so the composition root is the only place that changes. Extensions get text in and return cards out;
they do no I/O, and the application validates and writes what they return. `scripts/check-deps.sh` makes
`examples/*` depend on `studyfrog-ext` only.

## Consequences
+ Extensions are testable like any pure function, listable in the UI, and cannot reach the vault or database.
+ No `unsafe`, no plugin ABI to keep stable.
- Extensions are compiled in; adding one needs a rebuild. Users cannot install plugins.
- A function pointer cannot carry state or configuration. An extension that needs settings would have to read
  them through a new, explicit extension point.

## Alternatives
Trait objects: allowed by principle P5 but would put behaviour behind objects for no gain here. Dynamic loading:
needs `unsafe` or a WASM runtime; revisit if user-installable plugins become a goal.
