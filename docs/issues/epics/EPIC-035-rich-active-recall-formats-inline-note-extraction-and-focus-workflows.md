---
id: EPIC-035
type: epic
title: "Rich active-recall formats, inline note extraction, and focus workflows"
status: done
parents: []
labels: [m12, core, md, rehearsal, service, ui]
acceptance_criteria:
  - "Learners can author and rehearse inline Cloze deletion cards ({{c1::answer::hint}}) and visual Image Occlusion cards for diagrams and PDF figures"
  - "Markdown notes support inline flashcard and question extraction (such as Q:/A: or concept::definition blocks) that syncs linked study items without duplicating source context"
  - "Rehearsal and study workflows include a contrastive side-by-side drill for confused items and a built-in Pomodoro focus timer integrated with the Learning Diary"
---

## Context
Research across leading spaced-repetition and academic personal knowledge management (PKM) platforms (Anki, RemNote, Mochi, and Obsidian Spaced Repetition) highlights several high-impact workflows that bridge note-taking, visual diagram memorization, active recall, and deep focus:
1. **Cloze Deletions & Image Occlusion:** Two-sided flashcards (`???`) and free-response questions work well for discrete prompts, but learners studying definitions, formulas, code snippets, anatomical diagrams, or statistical path models need **Cloze Deletions** (`{{c1::target::hint}}`) and **Image Occlusion** (masking labels on an image to test each region individually).
2. **Zero-Friction Note-to-Card Extraction:** Students taking lecture or textbook notes in Markdown want to mark key definitions or questions inline within their notes and extract/sync them into rehearsal cards automatically while preserving bidirectional links to the parent note.
3. **Contrastive Interleaving & Focus Sessions:** When two or more concepts are repeatedly confused (retroactive/proactive interference), studying them side-by-side in a contrastive comparison drill resolves confusion much faster than isolated card flips. Meanwhile, a built-in Pomodoro/focus timer helps learners structure reading, note-taking, and rehearsal blocks while automatically logging focus sessions to the Learning Diary.

## Description
Deliver five high-value learning and study workflow features across StudyFrog:
1. **Inline Cloze Deletions (`STORY-0108`):**
   - Support `{{c1::answer}}` and `{{c1::answer::hint}}` syntax in cards and notes, generating independent or grouped cloze rehearsal prompts with masked `[...]` / `[hint]` rendering and typed or flip reveal.
2. **Image Occlusion Cards for Diagrams & PDF Figures (`STORY-0109`):**
   - Allow users to select an image in the vault or extract a figure from a PDF and draw rectangular occlusion masks over labels, supporting both *Hide All, Guess One* and *Hide One, Guess One* rehearsal modes.
3. **Inline Flashcard & Question Extraction from Markdown Notes (`STORY-0110`):**
   - Parse inline `Q: ... / A: ...` callouts or `Concept :: Definition` lines inside Markdown notes and offer a one-click `Extract / Sync study items` action that creates or updates linked cards/questions in the stack.
4. **Contrastive Side-by-Side Drill for Confused Items (`STORY-0111`):**
   - Provide a specialized rehearsal/drill mode that presents two or more frequently confused or linked items side-by-side, prompting the learner to articulate or match their distinguishing features.
5. **Pomodoro Focus Timer with Diary & Analytics Integration (`STORY-0112`):**
   - Add a non-intrusive top-bar Focus / Pomodoro timer (`25/5`, `50/10`, or custom work/break intervals) that tracks focused study time across reading, writing, and rehearsal, and offers a one-click Learning Diary reflection when a focus block finishes.

## Acceptance Criteria
- [x] Learners can author and rehearse inline Cloze deletion cards ({{c1::answer::hint}}) and visual Image Occlusion cards for diagrams and PDF figures
- [x] Markdown notes support inline flashcard and question extraction (such as Q:/A: or concept::definition blocks) that syncs linked study items without duplicating source context
- [x] Rehearsal and study workflows include a contrastive side-by-side drill for confused items and a built-in Pomodoro focus timer integrated with the Learning Diary

## Design Notes
Child stories:
- STORY-0108: Inline cloze deletions in flashcards and notes (estimate: 5)
- STORY-0109: Image occlusion cards for diagrams, slides, and PDF figures (estimate: 8)
- STORY-0110: Inline flashcard and question extraction from Markdown notes (estimate: 5)
- STORY-0111: Contrastive side-by-side drill for confused and interfering items (estimate: 3)
- STORY-0112: Pomodoro focus timer with automatic learning diary and session logging (estimate: 3)

Architecture guardrails:
- All vault files remain human-readable plain-text Markdown (`.md`); image occlusion masks are stored as normalized coordinates (`x, y, w, h` in `[0.0, 1.0]`) in item front-matter or a clean Markdown table block so the underlying image file is never mutated.
- Strictly adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Progress (2026-11-08)
All five stories done: STORY-0110 (inline extraction and sync), STORY-0108 (cloze), STORY-0112 (focus timer),
STORY-0111 (contrastive drill), STORY-0109 (image occlusion). Specs: `docs/specs/inline-extraction.md`,
`cloze.md`, `focus-timer.md`, `contrastive-drill.md`, `image-occlusion.md`. Each was checked by unit, property,
fixture-vault and egui harness tests and by screenshots of the real binary; none was used by a person in an
interactive window yet.

## Dependencies
EPIC-005 (Markdown and Front-Matter), EPIC-007 (Rehearsal Engine), EPIC-022 (PDF Documents), and EPIC-024 (Diary).

## Open Questions
None.
