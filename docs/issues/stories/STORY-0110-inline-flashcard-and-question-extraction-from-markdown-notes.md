---
id: STORY-0110
type: story
title: Inline flashcard and question extraction from Markdown notes
status: backlog
epic: EPIC-035
parents: [EPIC-035]
estimate: 5
labels: [m12, md, service, ui]
acceptance_criteria:
  - " studyfrog-md recognizes inline study-item blocks inside Markdown notes, including 'Concept :: Definition' descriptor lines and 'Q: ... / A: ...' callout blocks"
  - "The Note editor provides an 'Extract / Sync cards' action with a preview showing new and updated cards to be created in the note's stack"
  - "Extracted cards automatically carry a bidirectional wikilink back to the source note and heading and update idempotently on subsequent syncs without creating duplicates"
---

## Context
During lectures or textbook reading, students often prefer taking continuous structured notes in a single Markdown document rather than opening the `+ Create` modal for every individual term. In tools like RemNote and Obsidian-to-Anki, writing `Multiple Determiniertheit :: Ein Kriterium hängt fast nie nur von einer einzigen Ursache ab` or a `Q: ... / A: ...` block inline inside a note allows instant, idempotent extraction into scheduled flashcards linked back to the source note.

## Description
Build inline note-to-card extraction and synchronization:
1. **Inline Card & Question Parser (`crates/studyfrog-md/src/inline_extract.rs`):**
   - Scan a Markdown note body (ignoring fenced code blocks) for two ergonomic inline conventions:
     - **Concept–Descriptor lines:** `- Term :: Definition` or `Term :: Definition` (creates a `Doc::Card` with `front = Term`, `back = Definition`).
     - **Q&A blocks:** `Q: <prompt>` followed by `A: <answer>` (or `??: <question>` for `Doc::Question`).
   - Track the enclosing Markdown `# Heading` so extracted items inherit context and link back to `[[Note Title#Heading]]`.
2. **Idempotent Sync & Backlink Stamping (`crates/studyfrog-service`):**
   - Match extracted blocks against existing cards/questions in the stack by source note UID + normalized prompt/marker ID so re-running extraction updates modified definitions without duplicating items or resetting SRS schedules.
   - Automatically append a `Source: [[Note Title#Heading]]` reference and inherit the note's tags, subjects, and stack.
3. **Editor Preview & Sync Action (`crates/studyfrog-ui/src/view/editor.rs`):**
   - When editing a `Note` that contains extractable inline blocks, display an `"Extract N cards…"` pill in the editor header that previews the items and syncs them with one click (and full undo support).

## Acceptance Criteria
- [ ] studyfrog-md recognizes inline study-item blocks inside Markdown notes, including 'Concept :: Definition' descriptor lines and 'Q: ... / A: ...' callout blocks
- [ ] The Note editor provides an 'Extract / Sync cards' action with a preview showing new and updated cards to be created in the note's stack
- [ ] Extracted cards automatically carry a bidirectional wikilink back to the source note and heading and update idempotently on subsequent syncs without creating duplicates

## Design Notes
- Touchpoints:
  - `crates/studyfrog-md/src/inline_extract.rs`: pure extraction parser with round-trip and code-fence exclusion tests.
  - `crates/studyfrog-service/src/`: idempotent creation/update service call.
  - `crates/studyfrog-ui/src/view/editor.rs`: header badge and confirmation preview.
- Comply with P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-035.

## Open Questions
None.
