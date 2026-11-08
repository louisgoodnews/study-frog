---
id: STORY-0096
type: story
title: Item keys for querying
status: done
epic: EPIC-032
parents: [EPIC-032]
estimate: 5
labels: [m12, core, ui]
acceptance_criteria:
  - "Each stack, card, note, and question possesses a concise, sequential human-readable key (e.g. STACK_1, FLASHCARD_500, NOTE_12, QUESTION_3)"
  - "The search bar and query engine support filtering and direct navigation by item key"
  - "Item keys are displayed prominently in the item details view, editor header, and browser list"
---

## Context
StudyFrog currently identifies items using 26-character random alphanumeric NanoIDs (e.g. `01JBYQ00W9Y7A19B1X00000000`) and relative file paths. While random IDs prevent collision, they are virtually impossible for human learners to memorize, type into search bars, or reference in conversation. Providing short, memorable item keys (similar to issue tracking keys like `STACK_1` or `FLASHCARD_500`) significantly improves navigation ergonomics and enables rapid querying.

## Description
Implement human-readable item keys across StudyFrog:
1. Key Schema & Allocation:
   - Assign sequential keys based on item kind:
     - Stacks: `STACK_1`, `STACK_2`, ...
     - Flashcards: `FLASHCARD_1`, `FLASHCARD_2`, ...
     - Notes: `NOTE_1`, `NOTE_2`, ...
     - Questions: `QUESTION_1`, `QUESTION_2`, ...
   - Persist the allocated key in item frontmatter (`key: FLASHCARD_500`) or vault index, guaranteeing stability across app restarts and renames.
2. Query Engine & Search Integration:
   - Extend `studyfrog-core::query` to recognize key predicates (`key:FLASHCARD_500` or `key:STACK_1..10`).
   - Typing an exact key into the navbar or browser search bar directly navigates to or filters the matching item.
3. UI Presentation:
   - Display the item key as a badge in the item header, browser tables, and details view notebook tabs.
   - Provide a quick "Copy key" action for easy pasting into notes or wikilinks.

## Acceptance Criteria
- [x] Each stack, card, note, and question possesses a concise, sequential human-readable key (e.g. STACK_1, FLASHCARD_500, NOTE_12, QUESTION_3)
- [x] The search bar and query engine support filtering and direct navigation by item key
- [x] Item keys are displayed prominently in the item details view, editor header, and browser list

## Design Notes
- Crate touchpoints:
  - `crates/studyfrog-core/src/query.rs`: Add `Key(String)` term to query syntax and matching logic.
  - `crates/studyfrog-store/src/`: Maintain a counter in vault metadata to allocate monotonic sequence numbers per item kind.
  - `crates/studyfrog-ui/src/view/browser.rs` & `item_details.rs`: Render item key badge.
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Built:
- `core::item_key` (parse, normalise, next key, reading order) and the query field `key` (`=`, `!=`, `~`, `!~`, `IN`, `IS EMPTY`, `ORDER BY key`); `NOTE_012` equals `NOTE_12`.
- The key lives in front matter (`key: FLASHCARD_500`) as an unknown key (`studyfrog_md::extra`), so it survives renames and restarts. Store migration 24 adds `item.key`, `item.color` and `key_counter` (highest number handed out per prefix, so a deleted item's key is not reused).
- New stacks, cards, notes and questions get the next key of their kind when created (create dialog, quick card, stacks, imports). Existing items get keys only through the Library's "Assign keys…" with a confirmation; it is undoable like a bulk edit. Opening a vault never rewrites files.
- Search: an exact key typed into the top-bar search finds the item (stacks too, opened on the item page); in the Library's Standard search a word that is a key filters to that item; Query mode has `key`.
- Shown: a Key column in the library list (on by default), the key badge with "Copy key" in the item page's toolbar, and a Key row in the Details > Properties tab.

Verified: `crates/studyfrog-core/tests/item_keys.rs`, `crates/studyfrog-store/tests/item_keys.rs`, `crates/studyfrog-service/tests/item_keys.rs` (creation, rename + restart, assign and undo), `crates/studyfrog-ui/tests/item_keys_ui.rs` (real worker: confirm, assign, library and top-bar search, badge, Copy key); a screenshot of the Library screen with the Key column and "Assign keys…".

Not built (description only, not an acceptance criterion): the range form `key:STACK_1..10`. An import that replaces an existing file gives it a new key instead of keeping the old one.

## Dependencies
EPIC-032.

## Open Questions
None.
