# StudyFrog open points

Everything that was planned for phase M12 is built and released (`0.5.0` and `0.6.0`); what each release holds is in
**[FEATURES.md](FEATURES.md)** and the release notes under `docs/`. For live issue statuses see **[BOARD.md](BOARD.md)**.
These points are still open:

| Issue | What is open |
| :--- | :--- |
| [STORY-0114](docs/issues/stories/STORY-0114-psychophysical-typographic-measure-modular-hierarchy-and-tabular-numerals.md) | The criterion names a reading measure of 620 to 680 px; in StudyFrog's font that is about 100 characters, so the measure is about 70 characters (450 pt). The owner should accept the character-based width or ask for the pixel one. |
| [STORY-0124](docs/issues/stories/STORY-0124-interactive-cli-control-protocol-and-console-log-streaming-for-ai-agents.md) | The control protocol covers state, navigation, items, stacks, queries, rehearsal and exam runs, import and export, screenshots, logs and events. Calendar, board, diary, reports, achievements, settings, assistant, PDF and folders can only be reached with `nav`; there is no widget tree and no stack delete. |
| EPIC-036, EPIC-038 | In review because of the two stories above. |

Smaller gaps are listed in each story's Progress section and in the release notes: image occlusion has one schedule per
card, exams do not feed familiarity, deleting or moving an item raises no script event, scripting needs Unix, and the new
providers, the MCP server and client were never run against real services.
