# StudyFrog open points

Everything planned for phases M12 (`0.5.0`/`0.6.0`, including `EPIC-036`/`STORY-0114` and `EPIC-038`/`STORY-0124`), M13 (`0.7.0`), M14 (`0.8.0`), `EPIC-055` (local Ollama assistant), and v1.0.0 feature epics (`EPIC-056`–`EPIC-062`, plus `STORY-0185`–`0187` of `EPIC-063`) is complete; what each release holds is in **[FEATURES.md](FEATURES.md)** and the release notes under `docs/`. For live issue statuses see **[BOARD.md](BOARD.md)** (596 of 612 issues done, 16 open across `EPIC-063`, `EPIC-067`, and `EPIC-068` as of 2026-11-23).

These points are still open:

| Issue | Status | What is open |
| :--- | :--- | :--- |
| [EPIC-063](docs/issues/epics/EPIC-063-altklausur-and-mock-exam-simulation-lab.md) (`review`), [STORY-0207](docs/issues/stories/STORY-0207-exam-generation-item-format-selection-modal.md) (`backlog`) | `review` / `backlog` | Interactive modal to select mock exam item formats (open question, fill in the gaps, single select, multiple select, calculation, case scenario) and format-aware generation/grading. |
| [EPIC-067](docs/issues/epics/EPIC-067-performance-benchmarking-and-continuous-improvement.md), [STORY-0201](docs/issues/stories/STORY-0201-reference-runs-on-the-owners-machine-with-identity-recorded.md), [STORY-0203](docs/issues/stories/STORY-0203-budgets-replaced-by-measured-baselines.md) | `blocked` | Reference benchmark runs, 10-run idle spread measurement, and replacing provisional budgets in `BENCHMARKING.md` with measured baselines wait on the owner's local reference machine. |
| [STORY-0202](docs/issues/stories/STORY-0202-regression-rule-compare-two-runs-flag-only-real-large-changes.md), [TASK-0278](docs/issues/tasks/TASK-0278-reference-run-script-repeated-invocations-medians-and-spread-rule.md) | `review` | Regression comparison rule (`bench-compare`) and reference-run script (`bench-reference.sh`) are built; waiting on full runs on the owner's machine and the release-check documentation (`TASK-0282`). |
| [STORY-0204](docs/issues/stories/STORY-0204-frame-time-measured-in-the-running-window.md), [STORY-0205](docs/issues/stories/STORY-0205-allocations-in-render-loops-with-dhat-and-a-profile-of-the-slowest-screen.md), [TASK-0279](docs/issues/tasks/TASK-0279-interleaved-paired-runs-of-two-builds.md), [TASK-0282](docs/issues/tasks/TASK-0282-release-check-compares-the-reference-run-with-the-saved-baseline.md)–[TASK-0286](docs/issues/tasks/TASK-0286-profile-the-slowest-screen-and-record-it.md) | `backlog` | Interleaved paired bench runs, pre-release benchmark comparison checklist, in-window frame-time capture on a 10k-note vault, `dhat` render-loop allocation tests, and profiling the slowest screen. |
| [EPIC-068](docs/issues/epics/EPIC-068-classroom-simulation-with-ai-teacher-and-students.md) | `backlog` | Classroom simulation: local/remote-model lecture chat with simulated teacher and students, user participation, individual & group task assignments, turn-in and grading, and local session persistence. |

## Completed features in Phase M13 (0.7.0)

All planned epics, stories, and bug fixes for Phase M13 are built, verified, and released in `0.7.0`:
- **Universal Item Linking & Comments (`EPIC-039`, `STORY-0135`)**: Wikilink expansions, Comments notebook tab, search filters.
- **Intermittent Typed Active Recall (`EPIC-040`, `STORY-0136`)**: Setup sampling ratio, dynamic card view switching.
- **System-Driven Architecture & Decoupled Services (`EPIC-041`)**: Vault lifecycle driven surveys and scans.
- **Interactive AI Content Generation (`EPIC-042`, `STORY-0137`)**: Interactive intake interview prompts.
- **Navigation, Profile & File Management UX Overhaul (`EPIC-043`, `STORY-0138`, `STORY-0139`, `STORY-0140`)**: Overhauled profile/vault switchers, file navigator dialogue, in-rehearsal comments drawer.
- **Timetable Schedule Planner (`EPIC-044`, `STORY-0141`)**: Weekly recurring lectures and lessons with calendar projection.
- **Calendar Daily & Weekly Schedule Views (`EPIC-044`, `STORY-0142`)**: 7-day hourly grid and single-day agendas.
- **Macro Library Expansion & Escaping (`EPIC-045`, `STORY-0143`, `STORY-0144`)**: Extended macro tokens and `{#...}` escaping.
- **Markdown List Syntax (`EPIC-046`, `STORY-0145`)**: `-`, `*`, numbers, and `a)`, `b.`, `c.` list markers.
- **Breviceps Macrops Mascot Icon (`EPIC-047`, `STORY-0146`)**: Stylized vector-drawn rain frog icon across themes.
- **Floating Burger Navigation (`EPIC-047`, `STORY-0147`)**: Research report and collapsible floating navigation FAB.
- **ScriptRunner & Internal Automation API (`EPIC-048`, `STORY-0148`)**: In-app code editor, runner, stdout/stderr, and API spec.
- **Bug Fixes**: BUG-0016 (calibration baseline), BUG-0017 (bold text visibility), BUG-0018 (code block indentations), BUG-0019 (notes stack membership optional), and in-rehearsal item editing with Again queue repositioning.

## Completed features in Phase M14 (0.8.0)

| Epic / Story | Summary |
| :--- | :--- |
| **[EPIC-049](docs/issues/epics/EPIC-049-expanded-scripting-environment-dsl-job-scheduling-and-execution-logging.md)** | **Expanded Scripting Environment, DSL, Job Scheduling & Logging:** Rebrand ScriptRunner to trademark-safe FrogScript Studio, evaluate lightweight DSL embedding, implement scheduled cron jobs and event-driven triggers, and provide dedicated internal stdout and stderr logging channels. |
| ↳ **[STORY-0149](docs/issues/stories/STORY-0149-rename-scriptrunner-to-avoid-trademark-conflicts.md)** | **Rename ScriptRunner (Trademark Safety):** Rebrand ScriptRunner across UI, code identifiers, and documentation to FrogScript / FrogScript Studio to prevent conflicts with Adaptavist. |
| ↳ **[STORY-0150](docs/issues/stories/STORY-0150-in-app-code-execution-console-and-dsl-research.md)** | **In-App Code Execution Console & DSL Research:** Interactive in-app console execution and comprehensive research evaluating zero-dependency embedded DSLs (e.g. Rhai) alongside Python IPC. |
| ↳ **[STORY-0151](docs/issues/stories/STORY-0151-scripting-ad-hoc-runs-scheduled-jobs-and-event-driven-triggers.md)** | **Ad-Hoc Runs, Scheduled Jobs & Event-Driven Triggers:** UI and service management for one-off ad-hoc runs, recurring cron schedules (`scripts.toml`), and reactive lifecycle hook bindings (`vault_opened`, `item_created`, `rehearsal_finished`). |
| ↳ **[STORY-0152](docs/issues/stories/STORY-0152-dedicated-internal-stdout-and-stderr-execution-logging.md)** | **Dedicated Internal stdout and stderr Logging:** Distinct ring buffers and visual presentation separating standard output from error diagnostics, with stream filtering and file export. |
