---
id: STORY-0112
type: story
title: Pomodoro focus timer with automatic learning diary and session logging
status: backlog
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
- [ ] The top bar includes a compact, toggleable Focus / Pomodoro timer supporting standard presets (25/5, 50/10) and custom work/break durations
- [ ] The timer continues running across all screens (PDF reading, Note writing, Stack planning, and Rehearsal) with pause, resume, skip-break, and optional stack assignment
- [ ] Completing a focus block records the focused duration against any active calendar study session and offers a one-click prompt to log a Learning Diary reflection

## Design Notes
- Touchpoints:
  - `crates/studyfrog-ui/src/focus_timer.rs`: pure state transitions and preset definitions.
  - `crates/studyfrog-ui/src/view/topbar.rs`: compact pill and popover menu.
- Strictly enforce P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-035.

## Open Questions
None.
