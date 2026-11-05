---
id: STORY-0094
type: story
title: Create menu due date and priority assignment for stacks and other creatable items
status: done
epic: EPIC-031
parents: [EPIC-031]
estimate: 3
labels: [m11, ui]
acceptance_criteria:
  - "The priority field is available, visible, and editable when creating a stack in the create dialog"
  - "The due date and reminder fields are available and editable when creating non-stack items (cards, notes, and questions) as well as stacks"
  - "Item priority and due dates chosen during creation are persisted to item frontmatter and scheduled as calendar deadlines where applicable"
---

## Context
When creating new items via the StudyFrog creation modal (`crates/studyfrog-ui/src/view/modal_create.rs`), metadata inputs were artificially partitioned based on item kind:
- Stacks (`NewKind::Stack`) exposed tags, a "Mark as stack folder" checkbox, and due date/reminder pickers, but completely omitted the Priority selector. In fact, `crates/studyfrog-ui/src/modal.rs` explicitly forced `(None, None)` for priority and difficulty when creating stacks.
- Non-stack items (`Card`, `Note`, `Question`) exposed Priority and Difficulty, but completely omitted Due Date and Reminder selectors.

Learners planning study schedules often need to assign priorities to stacks (e.g. prioritizing an upcoming exam stack over general reading), and conversely assign due dates to individual notes or flashcard batches without requiring a stack wrapper.

## Description
Unify metadata inputs in the item creation workflow:
1. Priority on Stacks:
   - Enable the priority selector row in `modal_create.rs` when `f.kind == NewKind::Stack`.
   - Update `modal.rs` to propagate `Some(form.priority)` to `service.create_item(...)` for stacks.
   - Persist priority to `_stack.md` frontmatter via `studyfrog-service::create_more`.
2. Due Dates & Reminders on All Creatable Items:
   - Expose due date picker and reminder selection for cards, notes, and questions in addition to stacks.
   - When a due date is specified for a card, note, or question, write the due date to the item's frontmatter.
   - Register an associated deadline calendar event (`EventKind::Deadline`) in `studyfrog-service::track`.
3. UI Consistency:
   - Maintain a cohesive metadata layout in the creation modal, ensuring form fields align predictably across all item kind selections.

## Acceptance Criteria
- [x] The priority field is available, visible, and editable when creating a stack in the create dialog
- [x] The due date and reminder fields are available and editable when creating non-stack items (cards, notes, and questions) as well as stacks
- [x] Item priority and due dates chosen during creation are persisted to item frontmatter and scheduled as calendar deadlines where applicable

## Design Notes
- Files to touch:
  - `crates/studyfrog-ui/src/view/modal_create.rs`: Adjust `fields()` and `metadata_fields()` so Priority is rendered for Stacks, and Due Date / Reminder fields are available for all kinds.
  - `crates/studyfrog-ui/src/modal.rs`: Adjust `choices` logic so stack priority is preserved rather than discarded as `None`.
  - `crates/studyfrog-service/src/create_more.rs`: Ensure `StackDoc` serialization writes `priority` frontmatter when present.
  - `crates/studyfrog-service/src/track.rs`: Support registering deadlines for individual non-stack items when a due date is provided at creation.
- Keep files under 250 lines and verify with `crates/studyfrog-ui/tests/modal*.rs`.

## Dependencies
EPIC-031.

## Open Questions
None.
