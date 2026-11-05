---
id: EPIC-031
type: epic
title: Details tabs, create workflow, and CSV import for 0.4.1
status: done
parents: []
labels: [m11, ui, ext]
acceptance_criteria:
  - "The details view on the item page is organized into an accessible tabbed notebook layout with stack due date editing"
  - "The create dialog supports setting due dates and priorities across stacks and all other creatable items"
  - "Creating a note expands the modal to fill the window and defaults to write mode with shrink and resize capabilities"
  - "The importer seamlessly supports both CSV and TSV formats, handling Gemini AI prompt outputs without format conversion errors"
  - "The vault switcher menu renders clean vault folder names in borderless selectable rows with disambiguation and hover paths"
---

## Context
Following the 0.4.0 release, real-world usage and study sessions identified key usability bottlenecks and friction points in the core workflow:
1. The item details view side panel has accumulated numerous cards (properties, multi-stack memberships, familiarity metrics, incoming and outgoing links), resulting in an overcrowded panel that demands continuous scrolling. Additionally, stacks lacked any mechanism to view or update due dates after initial creation.
2. The item creation workflow lacked parity across item types: stacks were prevented from setting priority, while non-stack items (cards, notes, questions) could not specify due dates or reminders at creation time.
3. Creating markdown notes in the create dialog was constrained by split preview mode and fixed dialog widths, limiting the available space for composing long-form study notes.
4. AI flashcard generation outputs from Google Gemini or other LLMs frequently produce CSV text matching the prompt schema (`stack,kind,title,front,back,tags`), but the importer failed abruptly because it only accepted tab-separated values (TSV).

## Description
Deliver patch release 0.4.1 addressing these workflow improvements and bug fixes:
1. Details View Notebook Overhaul (STORY-0093):
   - Restructure the details panel into a tabbed notebook layout (Properties, Stacks & References, Familiarity & Metrics, Links & Graph).
   - Provide direct stack due date and reminder editing from the details view for stack items.
2. Unified Create Menu Metadata (STORY-0094):
   - Expose priority setting when creating stacks.
   - Expose due date and reminder pickers when creating cards, notes, and questions.
   - Persist frontmatter and schedule calendar deadlines accordingly.
3. Note Creation Full-Window Expansion (STORY-0095):
   - Automatically expand the create dialog to fill the available window when the note item kind is selected.
   - Default note editing to write mode rather than split preview.
   - Retain responsive resize dragging, shrink toggle, and kind switching.
4. Robust Delimited File Importer (BUG-0011):
   - Implement automatic delimiter detection (comma vs. tab) and quoted value parsing.
   - Support both AI prompt header format (`stack,kind,title,front,back,tags`) and simple 2/3 column formats (`front,back[,tags]`).

## Acceptance Criteria
- [x] The details view on the item page is organized into an accessible tabbed notebook layout with stack due date editing
- [x] The create dialog supports setting due dates and priorities across stacks and all other creatable items
- [x] Creating a note expands the modal to fill the window and defaults to write mode with shrink and resize capabilities
- [x] The importer seamlessly supports both CSV and TSV formats, handling Gemini AI prompt outputs without format conversion errors
- [x] The vault switcher menu renders clean vault folder names in borderless selectable rows with disambiguation and hover paths

## Design Notes
Child stories and bugs:
- STORY-0093: Details view notebook overhaul with tabs and stack due date editing (estimate: 5)
- STORY-0094: Create menu due date and priority assignment for stacks and other creatable items (estimate: 3)
- STORY-0095: Note creation modal full-window expansion and default write view (estimate: 3)
- STORY-0099: Simplify vault switcher menu (estimate: 3)
- BUG-0011: Importer only accepts TSV format and fails on CSV prompt outputs (estimate: 3)

Touchpoints across crates:
- `crates/studyfrog-ui`: `view/item_details.rs`, `view/modal_create.rs`, `view/modal_size.rs`, `modal.rs`, `msg.rs`, `editor.rs`
- `crates/studyfrog-service`: `track.rs`, `create_more.rs`, `xfer_csv.rs`
- `examples/studyfrog-ext-example`: `tsv.rs` (delimiter auto-detection & CSV parsing)
- Release target: version 0.4.1 (Milestone M11).

## Dependencies
EPIC-026 (StudyFrog 0.4.0 base features).

## Open Questions
None.
