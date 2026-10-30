# Architecture

## Crate graph

```
                    studyfrog-app
                          |
                    studyfrog-ui
   +------+------+------+-+----+--------+---------+--------+
   |      |      |      |      |        |         |        |
 store  rehearsal slm  report achieve  board   calendar    md
   |  \     |            |                                  |
   |   \    +--- srs ----+                                  |
   +----+--------------- core -----------------------------+
```

Textual rules (also in [ADR 0001](adr/0001-workspace-and-crate-layout.md)):

| Crate | Depends on | Responsibility |
|---|---|---|
| `core` | none | Pure data types, ids, arenas. No I/O. |
| `config` | core | Parse and render `studyfrog.toml` and `_stack.toml` ([ADR 0007](adr/0007-settings-in-toml-files.md)). |
| `md` | core | Markdown and front-matter parse/render. |
| `store` | core, md, config | SQLite and file layer; the only crate doing storage I/O. |
| `srs` | core | Scheduling functions. |
| `rehearsal` | core, srs | Run state machine, latency capture. |
| `slm` | core | Comparison trait and local runtime. |
| `report` | core, srs | Retention, latency distributions, forecasts. |
| `achieve` | core | Achievement rules and evaluation. |
| `board` | core | Kanban/scrum model and moves. |
| `calendar` | core | Session planning. |
| `service` | core, md, config, store, srs, rehearsal | Application layer: use cases that compose domain crates ([ADR 0006](adr/0006-application-service-layer.md)). |
| `ui` | service and domain crates | Rendering and event dispatch only. |
| `app` | ui | `main`, wiring, configuration. |

Rules: no cycles; `service` is the only place that wires several domain crates into a use case; domain crates never import `ui`/`app`; domain crates other than `store` and
`slm` do no I/O; sibling domain crates do not depend on each other except as listed.

## Data flow

```
 Markdown files --scan--> md (parse) --> core types --> store (index in SQLite)
                                              |
 UI event --> Msg --> update(State, Msg) -> (State, [Effect])
                                              |
        Effects executed at the edge (app/ui): store I/O, SLM call, clock read
        results come back as Msg; domain functions stay pure
```

- **Source of truth for content:** Markdown files. SQLite holds derived indexes (link graph,
  search) and history (reviews, run events, board, calendar). See ADR 0003.
- **Update loop:** state is plain data; `update` is a free function returning new state and a
  list of effect descriptions. Effects (disk, SLM, clock) are executed by the shell at the
  edge, so every domain decision is unit- and property-testable without mocks.
- **Hot paths:** rehearsal loop, SLM comparison and graph rendering are benchmarked
  ([BENCHMARKING](contributing/BENCHMARKING.md)); graph and run data use SoA/arena layouts.
- **Time:** domain functions take `now` as a parameter; latency uses a monotonic clock
  supplied by the shell, calendar uses wall-clock `chrono` types.

## Extensibility

Polymorphism is by trait at explicit seams only: `SlmRuntime`, `Clock`, `ContentStore`,
and (M8) extension points for card types and importers. Registries are plain data
(`Vec` of function pointers or ids), not object hierarchies.
