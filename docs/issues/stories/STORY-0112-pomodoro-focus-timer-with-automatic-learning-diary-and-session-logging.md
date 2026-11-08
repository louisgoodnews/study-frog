---
id: STORY-0112
type: story
title: Pomodoro focus timer with automatic learning diary and session logging
status: done
epic: EPIC-035
parents: [EPIC-035]
estimate: 3
labels: [m12, calendar, report, service, ui]
acceptance_criteria:
  - "The top bar includes a compact, toggleable Focus / Pomodoro timer supporting standard presets (25/5, 50/10) and custom work/break durations"
  - "The timer continues running across all screens (PDF reading, Note writing, Stack planning, and Rehearsal) with pause, resume, skip-break, and optional stack assignment"
  - "Completing a focus block records the focused duration against any active calendar study session and offers a one-click prompt to log a Learning Diary reflection"
---

## Context
StudyFrog's rehearsal engine tracks time spent flipping cards, but university study involves substantial time reading PDFs, summarizing lecture notes, extracting flashcards, and organizing stacks. A built-in, non-intrusive **Pomodoro / Focus Timer** in the top bar lets learners structure their entire study session (e.g., `25 min work / 5 min break` or `50 min work / 10 min break`) across all screens while automatically attributing focused minutes to the active stack and prompting a Learning Diary reflection when a cycle completes.

## Description
Implement an app-wide Focus / Pomodoro Timer in StudyFrog:
1. **Pure Timer State Machine (`crates/studyfrog-ui` / `crates/studyfrog-core`):**
   - Support `Phase::Work`, `Phase::ShortBreak`, and `Phase::LongBreak` (every 4th cycle), with presets (`25 / 5 min`, `50 / 10 min`, `90 / 15 min`, or custom minutes) and an optional target `StackId`.
   - Driven by the app's injected monotonic clock so all state transitions remain 100% deterministic and testable in headless UI tests.
2. **Top-Bar Pill & Controls (`crates/studyfrog-ui/src/view/topbar.rs`):**
   - Display a compact top-bar pill (e.g., `Focus 24:12 · Multivariate Verfahren`) when active, or a quiet timer icon when idle.
   - Clicking opens a lightweight popover to start, pause/resume, reset, switch preset, or pick the target stack.
3. **Diary & Calendar Session Integration:**
   - When a `Work` block finishes, raise an in-app notification toast (`"Focus session complete (25 min)"`) with one-click buttons to `Start break (5 min)` or `Log in Diary…` (pre-filling duration and stack).

## Acceptance Criteria
- [x] The top bar includes a compact, toggleable Focus / Pomodoro timer supporting standard presets (25/5, 50/10) and custom work/break durations
- [x] The timer continues running across all screens (PDF reading, Note writing, Stack planning, and Rehearsal) with pause, resume, skip-break, and optional stack assignment
- [x] Completing a focus block records the focused duration against any active calendar study session and offers a one-click prompt to log a Learning Diary reflection

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/focus_timer.rs`: pure state transitions and preset definitions.
  - `crates/studyfrog-ui/src/view/topbar.rs`: compact pill and popover menu.
- Strictly enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and verified:
- Pure state machine `crates/studyfrog-ui/src/focus_timer.rs` (work, short and long break, presets 25/5, 50/10,
  90/15, custom lengths, pause, resume, reset, skip break, tick on the injected clock); unit and property tests
  `crates/studyfrog-ui/tests/focus_timer.rs` (a block ends on time, pauses do not count, every fourth break is long).
- Top bar (`view/focus_bar.rs`): a stopwatch icon while idle, "Focus 24:12" / "Paused" / "Break" while active,
  a menu with start/pause/resume, skip break, reset, presets, custom lengths and the stack; the completion card
  with "Start break (N min)" and "Log in Diary…" (`focus_ui.rs`). Harness tests `crates/studyfrog-ui/tests/focus_ui.rs`
  (menu clicks, the pill on the PDF, editor, calendar, rehearsal and library screens while the time keeps counting,
  the card and the diary draft, the real worker recording the block); screenshots `--screen focus` and
  `--screen focus-done` looked at.
- Record: store migration 26 `focus_block`, `studyfrog_store::focus`, `studyfrog_service::focus::record_focus`
  (the study session occurrence overlapped most, local time); fixture tests `crates/studyfrog-service/tests/focus.rs`.
- Spec `docs/specs/focus-timer.md`; `docs/specs/storage-schema.md` v26.

Notes: the calendar does not yet show the recorded minutes on a session (they are stored and announced in the
notice). At a 900 px wide window the top bar was already crowded before this story (the Create button covered
the bell); the vault name is now cut short instead of overlapping, the narrowest layout is not otherwise reworked.

## Dependencies
EPIC-035.

## Open Questions
None.
