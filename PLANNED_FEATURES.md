# StudyFrog open points

Everything that was planned for phase M12 is built and released (`0.5.0` and `0.6.0`); what each release holds is in
**[FEATURES.md](FEATURES.md)** and the release notes under `docs/`. For live issue statuses see **[BOARD.md](BOARD.md)**.
These points are still open:

| Issue | What is open |
| :--- | :--- |
| [STORY-0114](docs/issues/stories/STORY-0114-psychophysical-typographic-measure-modular-hierarchy-and-tabular-numerals.md) | The criterion names a reading measure of 620 to 680 px; in StudyFrog's font that is about 100 characters, so the measure is about 70 characters (450 pt). The owner should accept the character-based width or ask for the pixel one. |
| [STORY-0124](docs/issues/stories/STORY-0124-interactive-cli-control-protocol-and-console-log-streaming-for-ai-agents.md) | The control protocol covers state, navigation, items, stacks, queries, rehearsal and exam runs, import and export, screenshots, logs and events. Calendar, board, diary, reports, achievements, settings, assistant, PDF and folders can only be reached with `nav`; there is no widget tree and no stack delete. |
| EPIC-036, EPIC-038 | In review because of the two stories above. |

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

## Upcoming features (Phase M14 / 0.8.0)

| Epic / Story | Summary |
| :--- | :--- |
| **[EPIC-049](docs/issues/epics/EPIC-049-expanded-scripting-environment-dsl-job-scheduling-and-execution-logging.md)** | **Expanded Scripting Environment, DSL, Job Scheduling & Logging:** Rebrand ScriptRunner to trademark-safe FrogScript Studio, evaluate lightweight DSL embedding, implement scheduled cron jobs and event-driven triggers, and provide dedicated internal stdout and stderr logging channels. |
| ↳ **[STORY-0149](docs/issues/stories/STORY-0149-rename-scriptrunner-to-avoid-trademark-conflicts.md)** | **Rename ScriptRunner (Trademark Safety):** Rebrand ScriptRunner across UI, code identifiers, and documentation to FrogScript / FrogScript Studio to prevent conflicts with Adaptavist. |
| ↳ **[STORY-0150](docs/issues/stories/STORY-0150-in-app-code-execution-console-and-dsl-research.md)** | **In-App Code Execution Console & DSL Research:** Interactive in-app console execution and comprehensive research evaluating zero-dependency embedded DSLs (e.g. Rhai) alongside Python IPC. |
| ↳ **[STORY-0151](docs/issues/stories/STORY-0151-scripting-ad-hoc-runs-scheduled-jobs-and-event-driven-triggers.md)** | **Ad-Hoc Runs, Scheduled Jobs & Event-Driven Triggers:** UI and service management for one-off ad-hoc runs, recurring cron schedules (`scripts.toml`), and reactive lifecycle hook bindings (`vault_opened`, `item_created`, `rehearsal_finished`). |
| ↳ **[STORY-0152](docs/issues/stories/STORY-0152-dedicated-internal-stdout-and-stderr-execution-logging.md)** | **Dedicated Internal stdout and stderr Logging:** Distinct ring buffers and visual presentation separating standard output from error diagnostics, with stream filtering and file export. |
