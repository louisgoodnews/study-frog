---
id: STORY-0100
type: story
title: "Move stack between parent stack and standalone after creation"
status: done
epic: EPIC-031
parents: [EPIC-031]
estimate: 3
labels: [m11, service, ui]
acceptance_criteria:
  - "Users can move a child stack out of its parent stack to become a standalone (top-level) stack from the stack page and the stack details view"
  - "Users can move a standalone or child stack into any valid parent stack (excluding itself and its own sub-stacks) from the stack page and the stack details view"
  - "Moving a stack preserves its items' primary stack association and spaced-repetition schedules while updating the vault overview, active selection, and open editor path"
---

## Context
Previously, a stack's hierarchy (whether it is a standalone top-level stack or nested inside a parent stack) was fixed at creation time unless the user manually moved directories in the folder tree. Users need a direct, discoverable way on the stack page and in the stack details view to promote a nested stack to a standalone stack or move a standalone/nested stack under another parent stack.

## Description
1. Service Layer (`studyfrog-service`):
   - Ensure `plan_move` does not treat moving a stack directory itself (`_stack.md` at `clean_src`) as a cross-stack item migration that drops the stack's own items' SRS schedules.
2. UI & State (`studyfrog-ui`):
   - Add `StackViewMsg::MoveStack(String, String)` and `StackViewMsg::Moved(Result<Box<(VaultInfo, String, String)>, String>)` in `stack_view_ui.rs` and `LibraryEffect::MoveStack(String, String)` in `library_bulk.rs` / `exec_library.rs`.
   - On the stack page (`view/stack_view.rs`), add a `"Move stack…"` submenu inside `"More actions"` offering `"Make standalone (top-level)"` (when the stack has a parent) and all valid parent stacks (excluding itself, its descendants, and its current parent).
   - On the stack details page (`view/item_details/tab_properties.rs`), display a `"Parent stack"` selector for stacks allowing the user to switch between `"None (standalone)"` and any valid parent stack.

## Acceptance Criteria
- [x] Users can move a child stack out of its parent stack to become a standalone (top-level) stack from the stack page and the stack details view
- [x] Users can move a standalone or child stack into any valid parent stack (excluding itself and its own sub-stacks) from the stack page and the stack details view
- [x] Moving a stack preserves its items' primary stack association and spaced-repetition schedules while updating the vault overview, active selection, and open editor path

## Design Notes
- Crate touchpoints:
  - `crates/studyfrog-service/src/folder_ops_plan.rs`
  - `crates/studyfrog-ui/src/stack_view_ui.rs`
  - `crates/studyfrog-ui/src/library_bulk.rs`
  - `crates/studyfrog-ui/src/exec_library.rs`
  - `crates/studyfrog-ui/src/view/stack_view.rs`
  - `crates/studyfrog-ui/src/view/item_details/tab_properties.rs`
- Release target: version 0.4.3.

## Dependencies
EPIC-031.

## Open Questions
None.
