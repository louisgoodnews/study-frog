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

## Spike results (M4, 2026-10-30)
Stack: eframe/egui 0.37 with `glow`, `wayland`, `x11`, `accesskit`; `egui_kittest` 0.37 for tests.
- **Builds:** cold build of the GUI dependency tree takes about 70 s; no extra system packages were needed.
- **Real window:** `studyfrog --smoke-frames 30` opened a window, drew 30 frames and exited cleanly (exit 0, 0.74 s)
  on the development machine (Linux, Wayland/X11 session). Run it under `xvfb-run` in CI.
- **Headless testing works well:** `egui_kittest` drives the real views without a window. Tests click buttons,
  type into the path field and read the accessibility tree.
- **Accessibility (partial):** AccessKit is enabled. An automated audit asserts that every button and text input
  on every screen has an accessible name; the path field is labelled via `labelled_by`.
- **Not verified (needs a person at a keyboard):** IME composition, real screen-reader output (for example Orca),
  focus order, high-DPI scaling. These remain the main risk for the editor (TASK-0040); the fallback stays iced.
- **API note:** egui 0.37 moved to `App::ui(&mut Ui, ..)` with panels shown inside the root `Ui`.
- **Decision stands:** egui/eframe.

