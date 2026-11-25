# StudyFrog 1.1.0

A visual release and a round of fixes. Most screens now use the violet-to-pink redesign (gradient headlines and cards) with the same controls and behaviour. The first-run setup wizard can download a suggested Ollama model after a click, and choice questions work in mock exams. Everything runs locally; no telemetry leaves the machine. The full list of changes is in `CHANGELOG.md` under `[1.1.0]`.

Read the [Known limits](#known-limits) before relying on a feature.

## What is new

### Redesign (EPIC-070, EPIC-073)

- Home, the stack page, the library overview, the rehearsal card and setup, the settings and help, the calendar, the board, the diary and achievements, the graph, the assistant and the classroom use the gradient headline and card styles (STORY-0214 to STORY-0230).
- Welcome, Folders, the item editor, the PDF viewer and the Homework planner (STORY-0232 to STORY-0236).
- The Match grid, the Prompts screen, the Console and FrogCanvas (STORY-0237 to STORY-0240).
- The mock exam setup, exam view and exam result, and the first-run setup wizard (STORY-0241, STORY-0242).
- New violet-to-pink palette tokens, the default accent is violet, and the six accents stay selectable (STORY-0226).

### Choice questions (EPIC-072)

- Single-choice and multiple-choice questions can be created, edited, viewed and rehearsed (STORY-0220 to STORY-0225).
- A `choice_grading` setting chooses all-or-nothing, Zapechelnyuk, or count-based partial credit (STORY-0231). Zapechelnyuk gives full credit when every correct option is chosen and no wrong one; this is an owner-decided deviation from the published 1/k rule.
- In a mock exam a choice item shows selectable options and no longer shows the correct option ticked (STORY-0241).

### First-run setup wizard (EPIC-069)

- The local-model step offers a confirmed download of the top suggested Ollama model that fits this computer, with its size and source shown. It starts only on a click; a failure shows Try again. Ollama checks the digests itself (STORY-0212).
- The MiniLM model and the catalog models keep their pinned SHA-256 checks (TASK-0298).

### Other

- `studyfrog-log` and `studyfrog-debug`, written in-house (STORY-0219).
- The window opens maximised when no size was asked for.

## Fixed

- Failed grades and per-item marks are in the danger colour; they were green (STORY-0240, STORY-0241).
- The mock exam's Start, Save as note and Finish and grade buttons sit above the questions, so they stay in view on a 1000 by 720 window (STORY-0241, STORY-0242).
- The GGUF catalog's digests and sizes for Qwen2.5-1.5B and Llama-3.2-1B, and the SmolLM2 file name (TASK-0298).

## Verification

- `scripts/check.sh` passes on `main` at the release commit (format, clippy with warnings as errors, the repo lint, line counts, dependency and impl rules, the issue board).
- `cargo test --workspace --no-fail-fast`: 2687 passed, 0 failed, before the final refactor. The gate's own test run after it passed.
- The exam choice fix was checked through the control protocol: a choice question shows radio options, nothing is marked correct, and `rehearse choose` then `rehearse submit` locks the answer and moves on.

## Known limits

- **Benchmarks not run for this release.** The baseline table in `docs/benchmarks/` was not refreshed on an idle machine (EPIC-067 is blocked on the owner's machine).
- **Heavy model tests not run.** `cargo test --release -p studyfrog-candle` needs the language model, which was not installed on the build machine. Word matching is the comparison fallback when no model is installed.
- **Screenshot checks at 1280 by 800 not done.** The display used for checks caps windows at 1200 by 675 pixels, so the redesigned screens were checked at their maximised size and at 1200 by 675 only.
- **Exam result view** after the exam still shows the correct options (by design); the exam choice flow was verified through the control protocol, not by an automated end-to-end test.
- **Export through the control protocol** needs a destination folder that the control command cannot set; export works from the Export dialog.
- **Known open issues** are in the issue board (`BOARD.md`).

## Install

Start the AppImage `StudyFrog-1.1.0-x86_64.AppImage` from a desktop session, open a vault, and check the SHA-256 printed next to the file.

## Upgrading from 1.0.0

The default accent changes from Blue to violet; choose Blue again under Settings > Appearance if you prefer it. Vault upgrade behaviour was not tested for this release, so back up a vault before opening it with 1.1.0.
