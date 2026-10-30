# M8: Extensibility and plugin boundaries

**Goal:** Explicit extension points and release readiness.

**Entry criteria:** M1-M7 exit met.

## Epics, stories, tasks

- **EPIC-016** Plugin boundaries
  - **STORY-0030** Extension traits
    - TASK-0063 Define extension points
    - TASK-0064 Registry as plain data
  - **STORY-0031** Example extension
    - TASK-0065 Example card type or importer
- **EPIC-017** Release hardening
  - **STORY-0032** Performance and packaging
    - TASK-0066 Profiling pass
    - TASK-0067 Linux packaging

- **EPIC-020** UI rework (added after the owner tried the app)
  - **STORY-0036** Top bar and sidebar
    - TASK-0093 Top bar, sidebar and menus
    - TASK-0094 Global search
    - TASK-0095 Notification list

## Exit criteria

- [x] Documented extension traits exist for card types and importers
- [x] An example extension is built without modifying core crates
- [x] Profiling pass done; all benchmarks within budget
- [ ] Linux package (AppImage or Flatpak) builds from CI (the AppImage builds and starts locally; the CI workflow has not run yet)

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
