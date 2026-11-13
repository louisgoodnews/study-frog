---
id: STORY-0135
type: story
title: "Comments for items"
status: done
epic: EPIC-039
parents: [EPIC-039]
estimate: 5
labels: [m12, core, md, store, ui]
acceptance_criteria:
  - "Vault items (stacks, cards, questions, and notes) support timestamped comments with author metadata in YAML front-matter (comments: [{ id, author, created, text }])"
  - "The Item Details notebook provides a dedicated Comments tab/section displaying chronological comments, author badges (learner or agent), and controls to add, edit, or delete comments"
  - "Comments are indexed in SQLite, searchable via query filters (e.g. has:comments, comment:<text>), and accessible through CLI control protocol and MCP tools"
---

## Context
While studying and reviewing materials, learners frequently encounter cards, questions, or notes where they want to record personal study hints, mnemonics, clarification questions, or reminders without modifying the core question prompt, answer text, or note body.

Furthermore, autonomous AI agents and assistants (operating through the CLI control protocol or MCP server) need an ergonomic, non-destructive mechanism to attach explanations, mnemonics, pronunciation tips, or grading feedback to specific items. A structured comment system attached to each item bridges human reflection and AI assistance while keeping the vault's Markdown source human-readable.

## Description
Deliver an end-to-end comment system for StudyFrog items:

1. **Front-Matter Schema & Serialization (`studyfrog-core`, `studyfrog-md`):**
   - Extend item metadata with an optional `comments` list in YAML front-matter:
     ```yaml
     comments:
       - id: "c_01j7xyz"
         author: "user" # or "agent", "gemini", custom name
         created: 1731140000000
         text: "Remember to verify boundary conditions when the denominator is zero."
     ```
   - Ensure comment timestamps, IDs, and markdown formatting are preserved during note/card serialization and roundtripping.
2. **SQLite Indexing & Query Filters (`studyfrog-store`, `studyfrog-core`):**
   - Index comments in an `item_comment` SQLite table (`id`, `item_uid`, `author`, `created_at`, `text`).
   - Extend query parsing and evaluation to support comment filtering:
     - `has:comments` (filter items that have at least one comment).
     - `comment:"keyword"` (full-text or substring match on comment content).
     - `comment-author:user` / `comment-author:agent`.
3. **UI Comments Notebook Tab (`studyfrog-ui`):**
   - Add a `Comments` tab to the Item Details side notebook (`crates/studyfrog-ui/src/view/item_details/tab_comments.rs`).
   - Render comment threads as styled timeline cards displaying author badge, relative timestamp, and formatted Markdown text.
   - Provide an inline text input with an "Add comment" button, edit toggle, and confirmation on deletion.
   - Show a comment count badge on the Comments tab header when comments are present.
4. **CLI & MCP Tooling Integration (`studyfrog-app`, `studyfrog-service`):**
   - Add CLI control protocol commands: `studyfrog ctl comment add <item_key_or_uid> "<text>"`, `studyfrog ctl comment list <item_key_or_uid>`, `studyfrog ctl comment delete <comment_id>`.
   - Expose corresponding MCP server tools: `vault_add_comment`, `vault_list_comments`.

## Acceptance Criteria
- [x] Vault items (stacks, cards, questions, and notes) support timestamped comments with author metadata in YAML front-matter (comments: [{ id, author, created, text }])
- [x] The Item Details notebook provides a dedicated Comments tab/section displaying chronological comments, author badges (learner or agent), and controls to add, edit, or delete comments
- [x] Comments are indexed in SQLite, searchable via query filters (e.g. has:comments, comment:<text>), and accessible through CLI control protocol and MCP tools

## Design Notes
- **Storage Strategy:** Comments live directly in the Markdown front-matter to adhere to StudyFrog's file-first philosophy; files copied to other systems or synced via Git retain all annotations without requiring a separate database dump.
- **SQLite Index:** The store parses front-matter comments during vault indexing for instant search, filtering, and aggregate queries.
- **Design Constraints:**
  - Strict compliance with P1 (no inherent `impl` blocks) and P2 (maximum 250 lines per Rust file).
  - UI components must follow the perceptual OKLCH color palette and low-entropy typography guidelines.

## Dependencies
- EPIC-005 (Markdown and front-matter).
- EPIC-021 (Knowledge workspace & Item details notebook).
- EPIC-031 (Details tabs layout).
- EPIC-038 (CLI control protocol).

## Open Questions
- Should comments be displayable as an optional expandable drawer during rehearsal and exam modes (e.g. "View notes & comments")? -> Resolved: Yes. Tracked in STORY-0140 (Expandable comments drawer in rehearsal and exam modes).
