---
id: STORY-0093
type: story
title: Details view notebook overhaul with tabs and stack due date editing
status: done
epic: EPIC-031
parents: [EPIC-031]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "The details view side column on the item page displays a tabbed notebook layout organizing item details across distinct tabs"
  - "Fields within the notebook tabs (tags, subjects, priority, difficulty, status, and stack references) remain mutable and interactive"
  - "Users can view, set, modify, or clear the due date and reminders of an existing stack directly from the stack details view tab"
  - "Keyboard navigation and accessibility labels support switching tabs and editing tab fields"
---

## Context
In StudyFrog 0.4.0, the item editor provides a side panel (`crates/studyfrog-ui/src/view/item_details.rs`) showing metadata and relational context for the active document. As features grew (supporting multiple stack memberships, per-stack familiarity scores, PDF source backlinks, tags, subjects, priority, difficulty, and status), all information was rendered sequentially in a single vertical stack of cards (`Details`, `item_stacks`, `item_fam`, `Links to`, `Linked from`).

This created visual clutter and heavy scrolling friction during study and editing sessions. Moreover, when inspecting a stack (`_stack.md`), users had no way to view or edit the stack's target due date or reminder settings, which could previously only be configured during initial stack creation.

## Description
Overhaul the details view side panel into a clean, tabbed notebook interface:
1. Tabbed Organization:
   - Partition details into distinct, purpose-focused tabs:
     - **Properties**: Kind badge, tags input, subject selectors, priority selector, difficulty selector, workflow status selector, PDF source backlink, timestamps (created / modified). For stacks, render stack deadline controls (due date picker and reminder selector).
     - **Stacks**: Primary stack, additional stack memberships (`also_in`), "Add to stack" button, and stack removal actions.
     - **Familiarity**: Retention metrics, familiarity score, review history count, average accuracy, and memory state.
     - **Links**: Outgoing wikilinks ("Links to") and incoming backlinks ("Linked from") with navigation actions.
2. Stack Due Date and Reminder Editing:
   - For `Doc::Stack` documents, display the current due date and reminder interval in the Properties tab.
   - Allow setting, updating, or clearing the due date using date picker controls.
   - Persist changes to stack metadata and update the associated calendar deadline event (`EventKind::Deadline`) via `studyfrog-service::track`.
3. Accessibility & Usability:
   - Provide clear tab buttons with accessible labels, keyboard focus indicators, and persistent active tab selection across item reloads.

## Acceptance Criteria
- [x] The details view side column on the item page displays a tabbed notebook layout organizing item details across distinct tabs
- [x] Fields within the notebook tabs (tags, subjects, priority, difficulty, status, and stack references) remain mutable and interactive
- [x] Users can view, set, modify, or clear the due date and reminders of an existing stack directly from the stack details view tab
- [x] Keyboard navigation and accessibility labels support switching tabs and editing tab fields

## Design Notes
- Files to touch:
  - `crates/studyfrog-ui/src/view/item_details.rs`: Refactor into tabbed notebook layout, introducing tab state tracking (`DetailsTab::Properties`, `DetailsTab::Stacks`, `DetailsTab::Familiarity`, `DetailsTab::Links`).
  - `crates/studyfrog-ui/src/msg.rs`: Add UI actions/messages for setting stack due dates and reminder updates if needed.
  - `crates/studyfrog-ui/src/editor.rs`: Handle stack due date persistence and sync with vault metadata.
  - `crates/studyfrog-service/src/track.rs`: Provide helper function for updating stack deadline events.
- Keep `item_details.rs` under the 250-line file limit by modularizing helper functions or sub-tab renderers if necessary.
- Ensure all tests in `crates/studyfrog-ui/tests/` continue to pass.

## Dependencies
EPIC-031.

## Open Questions
None.
