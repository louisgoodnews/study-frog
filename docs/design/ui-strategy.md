# StudyFrog UI strategy

Status: proposal, 2026-11-27. Derived from [ui-ux-learning-evidence-report](../research/ui-ux-learning-evidence-report.md). Mockups: [mockups/index.html](mockups/index.html). Nothing here is built yet; each decision below names the story it would become.

## 1. Goal

A screen should make the next learning action obvious and keep the learner's attention on it. Visual richness (the redesign of EPIC-070 and the violet-to-pink palette of STORY-0226) stays, but it serves that goal and never competes with it.

## 2. Principles

1. **One task per screen.** Each screen has one primary action. Other information is a disclosure, not a panel beside the task.
2. **Recall before reveal.** An item asks for an attempt before it shows the answer.
3. **Feedback that teaches.** The feedback panel gives: whether it was right, the correct answer, the reason, the source, and the next step. It has no points or streak pressure.
4. **Motion only where it shows a procedure.** Short, optional, and off under reduced motion.
5. **Progress leads to action.** Any progress view ends in "study this next, because...".
6. **Touch-ready by default.** Targets are 44 px on touch-capable layouts; keyboard shortcuts stay as the fast path.
7. **Claims are tested, not assumed.** Any UI change that claims a learning benefit is measured in the app first (section 7).

## 3. Information architecture

Top level, in a left rail (the rail pattern of STORY-0227/0230):

| Rail item | Purpose | Primary action |
|---|---|---|
| Today | What to study next and why | Start the suggested session |
| Rehearse | Recall sessions | Answer the current item |
| Items | Browse and edit items | Open an item |
| Graph | Link map, with the list alternative | Open an item from the map or list |
| Assistant | Model-backed help | Ask a question |
| Settings | Preferences, grouped by domain | Change a setting |

Rule: a screen never shows two primary actions. Secondary actions go in a menu or under a disclosure.

## 4. Screen specifications

### 4.1 Rehearsal card (mockup: `rehearsal.html`)
- Prompt and answer area occupy the centre; nothing else competes at the same weight.
- Answer first: a single text field or the choice list (number keys 1 to 9 select, Enter submits).
- Feedback appears in the same place after submit: verdict, correct answer, reason, source link, then rating buttons (Again, Hard, Good, Easy) with the suggested rating marked.
- No timer by default. A progress line shows the session, not a score.
- Stories: STORY-0217, STORY-0222 (choice rendering), STORY-0234.

### 4.2 Match debrief (mockup: `debrief.html`)
- Shows first-try accuracy and, once scheduled, retained accuracy from the delayed re-test.
- A confusion summary lists the pairs confused most, each with a one-tap drill.
- One primary action: "Drill missed items (N)". The delayed re-test is scheduled, not forced.
- Stories: the Match debrief work in progress (match_victory), plus a delayed re-test story to be written.

### 4.3 Today and navigation shell (mockup: `shell.html`)
- Left rail with the six items above; the current item is marked by the rail highlight and a text label, not colour alone.
- "Today" shows the suggested next session, the reason (for example, "12 items are due, 3 were missed yesterday"), and a single start button.
- Progress is a short list of actions, not a grid of charts. Charts move to Reports.
- Stories: STORY-0227/0230 rails (done for Settings, Graph and Classroom; this extends them to the shell), plus a Today story to be written.

### 4.4 Settings
- Grouped by domain, as now; the tab rail (STORY-0227) stays.
- Every setting has a one-line effect description. No setting depends on a colour alone.

## 5. Visual system

- Keep the OKLCH tokens and APCA contrast targets from the aesthetics note.
- Both light and dark modes, following the system setting by default; the user can override. No claim is made that either mode improves learning.
- Focus rings are always visible and meet the contrast target in both modes.
- Reduced motion: transitions are removed, not shortened, when the system asks for it.

## 6. Accessibility and touch

- Minimum target 44 px on touch-capable layouts (AAA, criterion 2.5.5), and at least 24 px everywhere (AA, criterion 2.5.8).
- Every control has a visible text label and a hover or long-press explanation (already enforced by the tooltip test).
- Keyboard: the full flow works without a mouse; shortcuts are listed in Help.
- Screen reader: the graph's list view stays as the accessible alternative to the canvas.

## 7. Measurement

Claims are tested with an opt-in, local protocol:
- Compare the single-task rehearsal layout with the current one on the same item set, using delayed recall (not immediate recall) as the outcome.
- Track the Today suggestion: did the learner start the suggested session, and did delayed recall improve?
- Record time-to-first-answer and error rate as usability measures.
- Data stays in the vault; nothing leaves the machine.

Results are reported in the release notes before any claim is made in the UI or docs.

## 8. Roadmap of stories

| Order | Story (to write) | Depends on | Evidence |
|---|---|---|---|
| 1 | Single-task rehearsal card with recall-first feedback | STORY-0217, STORY-0222 | Testing and feedback findings |
| 2 | Delayed re-test in Match debrief | Match mode | Retention, testing effect |
| 3 | Today screen with a suggested session | Rail shell | Progress-to-action finding |
| 4 | 44 px touch targets and touch layout | EPIC-084 | WCAG 2.5.5 and 2.5.8 |
| 5 | Local comparison protocol (section 7) | 1 and 2 | Measurement rule |
| 6 | Reduced-motion handling for all transitions | Redesign screens | Motion finding |

Each story lists its acceptance criteria when it is written, and each claim in its description cites the research note.

## 9. What is not proposed

- No points, leaderboards or streak penalties on the learning path.
- No learner-style labels (visual, auditory) that switch the UI. Modality choice is the user's, and is covered by EPIC-082.
- No dashboard that displays data without a next action.
- No dark-mode-as-learning-aid claim.
