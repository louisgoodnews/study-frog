---
id: EPIC-039
type: epic
title: Universal item linking between stacks, cards, questions, answers, and notes
status: backlog
parents: []
labels: [m12, core, md, store, graph, ui]
acceptance_criteria:
  - "The Markdown engine and link resolver support universal linking between all item types (stacks, cards, questions, answers, and notes) using item keys, titles, paths, and sub-item anchors (e.g. #back, #answer, #heading)"
  - "The SQLite index tracks bidirectional links across all item kinds, powering backlinks, reference queries, and a multi-kind visual knowledge graph"
  - "The UI renders kind-badged link chips with target color accents, hover preview popovers, an editor [[ autocomplete picker, and an enhanced Relations notebook tab"
---

## Context
StudyFrog's existing wikilink implementation (`studyfrog-md::links` and `tab_relations.rs`) is primarily oriented around linking note documents by title (`[[Note Title]]`). However, effective learning involves deep interconnections across all learning entities in a vault:
- A practice **Question** needs to reference a specific definition in a **Note** or the reference answer in another **Question**.
- A **Flashcard** prompt or answer needs to link directly to related **Flashcards** (e.g., contrasting concept pairs) or link back to a parent or associated **Stack**.
- An **Answer** should be directly targetable (e.g., linking specifically to a card's back or a question's reference rubric).
- The knowledge graph and item details notebook should reveal the full web of connections across stacks, cards, questions, answers, and notes, rather than only displaying note-to-note graphs.

## Description
Deliver an end-to-end item linking framework with rich logic, indexing, and visual representation across StudyFrog:

1. **Universal Item Addressing & Sub-Item Anchor Linking (`studyfrog-core`, `studyfrog-md`):**
   - Support linking across all item kinds using:
     - **Item Keys:** `[[STACK_1]]`, `[[FLASHCARD_42]]`, `[[QUESTION_7]]`, `[[NOTE_12]]` (with optional display label `[[FLASHCARD_42|Custom Label]]`).
     - **Titles and Aliases:** `[[Item Title]]`.
     - **Sub-Item Part Anchors:** Link directly to specific sections or answer components:
       - Card back / answer: `[[FLASHCARD_42#back]]` or `[[Card Title#back]]`.
       - Question reference answer: `[[QUESTION_7#answer]]` or `[[QUESTION_7#rubric]]`.
       - Note headings: `[[NOTE_12#Heading]]`.
   - Support optional explicit relationship metadata in YAML front-matter:
     - `links: [FLASHCARD_10, NOTE_5]`
     - `relations: [{ target: "QUESTION_3", kind: "prerequisite" }]`
2. **Bidirectional Link Indexing & Knowledge Graph (`studyfrog-store`, `studyfrog-graph`, `studyfrog-service`):**
   - Store all extracted item-to-item and item-to-anchor links in the SQLite index (`item_link` table with `source_uid`, `target_uid`, `target_anchor`, `relation_kind`).
   - Query backlinks for any item kind: find which cards, questions, notes, or stacks link to a target item or specific answer.
   - Upgrade the knowledge graph (`studyfrog-graph` and `view/graph.rs`) to visualize multi-kind nodes with distinct shapes, badges, and color styling for Stacks, Cards, Questions, and Notes.
3. **Visual Representation & Ergonomic UI Workflows (`studyfrog-ui`):**
   - **Kind-Badged Link Chips:** Render links inside Markdown previews, editor fields, and rehearsal views as styled chips with kind icons (Stack icon, Card icon, Question icon, Note icon), key badges, and target item color accents.
   - **Hover Preview Popovers:** Hovering over any item link shows a floating preview card with the target prompt, answer, or note excerpt without navigating away.
   - **Editor Autocomplete Picker (`[[`):** Typing `[[` inside any text field opens an interactive fuzzy-search dropdown listing stacks, cards, questions, and notes with type filtering.
   - **Enhanced Relations Notebook Tab:** Upgrade the Details Relations tab to categorize connections: *Parent Stack*, *Referenced Stacks*, *Links to Items & Answers*, *Linked from (Backlinks)*, and *Graph Neighborhood*.
   - **Rehearsal Context Links:** Rehearsal and Exam cards display related item pills allowing instant jump or side-by-side inspection of referenced items or answers.

## Acceptance Criteria
- [ ] The Markdown engine and link resolver support universal linking between all item types (stacks, cards, questions, answers, and notes) using item keys, titles, paths, and sub-item anchors (e.g. #back, #answer, #heading)
- [ ] The SQLite index tracks bidirectional links across all item kinds, powering backlinks, reference queries, and a multi-kind visual knowledge graph
- [ ] The UI renders kind-badged link chips with target color accents, hover preview popovers, an editor [[ autocomplete picker, and an enhanced Relations notebook tab

## Design Notes
Touchpoints across crates:
- `crates/studyfrog-core`: item link types, anchor kinds (`Front`, `Back`, `Answer`, `Rubric`, `Section`), and relation kinds.
- `crates/studyfrog-md`: link parser extensions for key detection and `#back` / `#answer` anchor parsing.
- `crates/studyfrog-store`: schema migrations for universal item links and bidirectional indexing.
- `crates/studyfrog-graph`: multi-kind graph nodes and edge rendering.
- `crates/studyfrog-service`: link resolution queries and hover preview loaders.
- `crates/studyfrog-ui`: kind-badged chip rendering, hover popovers, `[[` autocomplete widget, and Relations tab overhaul.

Architecture rules:
- Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
EPIC-005 (Markdown and front-matter), EPIC-006 (Storage), EPIC-009 (Editor and graph), and EPIC-032 (Item keys).

## Open Questions
None.
