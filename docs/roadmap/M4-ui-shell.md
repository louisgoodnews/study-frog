# M4: UI shell

**Goal:** First usable desktop app: browse, edit, rehearse.

**Entry criteria:** M3 exit met; ADR 0002 accepted.

## Epics, stories, tasks

- **EPIC-008** UI shell
  - **STORY-0016** App frame
    - TASK-0036 eframe window and navigation
    - TASK-0037 Elm-style update loop and effects
  - **STORY-0017** Stack browser and rehearsal screen
    - TASK-0038 Stack browser
    - TASK-0039 Rehearsal screen
    - TASK-0073 Run setup screen (plan picker)
- **EPIC-009** Editor and graph
  - **STORY-0018** Markdown editor and preview
    - TASK-0040 Editor widget
    - TASK-0041 Preview renderer
  - **STORY-0019** Link graph view
    - TASK-0042 Force-directed layout (SoA)
    - TASK-0043 Graph render benchmark

## Exit criteria

- [x] App launches and opens a vault
- [x] User can create a stack, write a card, rehearse it with the keyboard, and see it rescheduled
- [x] Markdown editor with preview works
- [x] Link graph layout and projection for 5k nodes take ~4 ms per frame with zero allocation (painting cost in egui itself not measured headlessly)
- [x] `update` layer has unit tests; UI crate holds no business logic

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
