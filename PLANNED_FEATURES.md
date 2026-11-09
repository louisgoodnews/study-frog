# StudyFrog open points

Everything that was planned for phase M12 is built and released (`0.5.0` and `0.6.0`); what each release holds is in
**[FEATURES.md](FEATURES.md)** and the release notes under `docs/`. For live issue statuses see **[BOARD.md](BOARD.md)**.
These points are still open:

| Issue | What is open |
| :--- | :--- |
| [STORY-0114](docs/issues/stories/STORY-0114-psychophysical-typographic-measure-modular-hierarchy-and-tabular-numerals.md) | The criterion names a reading measure of 620 to 680 px; in StudyFrog's font that is about 100 characters, so the measure is about 70 characters (450 pt). The owner should accept the character-based width or ask for the pixel one. |
| [STORY-0124](docs/issues/stories/STORY-0124-interactive-cli-control-protocol-and-console-log-streaming-for-ai-agents.md) | The control protocol covers state, navigation, items, stacks, queries, rehearsal and exam runs, import and export, screenshots, logs and events. Calendar, board, diary, reports, achievements, settings, assistant, PDF and folders can only be reached with `nav`; there is no widget tree and no stack delete. |
| EPIC-036, EPIC-038 | In review because of the two stories above. |

## Upcoming features (Phase M13 / 0.7.0)

| Epic / Story | Summary |
| :--- | :--- |
| **[EPIC-039](docs/issues/epics/EPIC-039-universal-item-linking-between-stacks-cards-questions-answers-and-notes.md)** | **Universal Item Linking:** Cross-link any items (stacks, cards, questions, answers, and notes) using keys, titles, and sub-item anchors (e.g. `#back`, `#answer`); bidirectional SQLite indexing; kind-badged link chips; hover preview popovers; `[[` editor autocomplete; and enhanced Relations notebook tab. |
| ↳ **[STORY-0135](docs/issues/stories/STORY-0135-comments-for-items.md)** | **Comments for Items:** Timestamped user and AI agent comments in YAML front-matter, SQLite indexing, search query filters (`has:comments`, `comment:<text>`), Item Details notebook Comments tab, and CLI/MCP controls. |
| **[EPIC-040](docs/issues/epics/EPIC-040-intermittent-typed-active-recall-and-flexible-rehearsal-presentation.md)** | **Intermittent Typed Active Recall:** Rehearsal setup sampling setting to randomly require typed written recall across study items, reducing typing fatigue while preventing familiarity illusions; notes remain readable reference cards. |
| ↳ **[STORY-0136](docs/issues/stories/STORY-0136-make-typed-answers-random-per-selection-during-rehearsal.md)** | **Random Typed Answers per Selection:** Setup sampling ratio, deterministic queue presentation assignment, and dynamic card view switching between typed submission and flip reveal. |
| **[EPIC-041](docs/issues/epics/EPIC-041-modularity-system-driven-architecture-and-decoupled-background-services.md)** | **System-Driven Architecture & Decoupled Services:** Redesign application processes so services (e.g. file surveys, index polling) are driven by vault lifecycle hooks and background routines rather than triggered lazily by UI screen mounts. |
| **[EPIC-042](docs/issues/epics/EPIC-042-interactive-ai-content-generation-and-parameter-guided-prompt-workflows.md)** | **Interactive AI Content Generation:** Prompt library workflows where AI models conduct an interactive intake interview (relevance, language, difficulty, style) before generating study items. |
| ↳ **[STORY-0137](docs/issues/stories/STORY-0137-interactive-parameter-guided-content-generation-prompts.md)** | **Interactive Intake Prompts:** Multi-turn file extraction prompts guiding users through content scope, difficulty, and format choices with clean CSV output. |
| **[BUG-0016](docs/issues/bugs/BUG-0016-metacognitive-calibration-accuracy-baseline-and-default-item-difficulty-priority.md)** | **Calibration Accuracy Baseline & Default Difficulty/Priority:** Calculate a realistic accuracy baseline in metacognitive calibration factoring in repetition history, latency vs. word count, and ratings; enforce default difficulty Hard and priority Medium on all item creation paths. |

Smaller gaps are listed in each story's Progress section and in the release notes: image occlusion has one schedule per
card, exams do not feed familiarity, deleting or moving an item raises no script event, scripting needs Unix, and the new
providers, the MCP server and client were never run against real services.
