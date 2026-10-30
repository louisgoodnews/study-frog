# ADR 0002: GUI framework

- Status: **Accepted** (2026-10-30)
- Date: 2026-10-30

## Context
Needs: native Linux, low overhead, custom drawing (graph view, calendar, board drag/drop),
text editing for Markdown, no web runtime. Constraint: functional style, no OOP,
no allocation in render loops.

## Options
| | egui/eframe | iced | Tauri/Electron |
|---|---|---|---|
| Model | immediate mode | retained, Elm architecture | web view |
| Custom painting (graph) | easy, direct `Painter` | canvas widget, more ceremony | JS |
| Fit with "state + free functions" | good: UI is `fn(&State, &mut Ui)` | very good: `update`/`view` are pure | poor |
| Text editing / rich widgets | adequate, improving | adequate | best (web) |
| Per-frame cost | re-runs UI every frame; cheap if tuned | diff-based | heavy |
| Accessibility | partial (AccessKit) | limited | good |

## Decision (proposed)
**egui/eframe.** Immediate mode makes the graph, board and calendar straightforward custom
painting and keeps UI code as plain functions over borrowed state. We impose an Elm-style
discipline ourselves: `update(State, Msg) -> (State, Vec<Effect>)` is pure and lives outside
`ui` rendering code, so the framework is replaceable.

Tauri/Electron are rejected: a web runtime contradicts the performance and footprint goals.

## Risks
- Accessibility and IME maturity in egui. Mitigation: spike in M4; fallback is iced, which the
  pure `update` layer keeps feasible.
- Per-frame rebuild cost on large vaults. Mitigation: virtualised lists, benchmark in M4.

## Review trigger
Revisit if M4's editor spike fails the acceptance criteria in STORY-0018.
