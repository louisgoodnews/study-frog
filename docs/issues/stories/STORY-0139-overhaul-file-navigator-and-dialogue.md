---
id: STORY-0139
type: story
title: Overhaul of file navigator and in-app file dialogue
status: done
epic: EPIC-043
parents: [EPIC-043]
estimate: 5
labels: [m12, ui, ux]
acceptance_criteria:
  - "The file navigator (Folders screen) provides a modern tree explorer with indent guidelines, file-type icons (Markdown, PDF, image, stack), search filtering, and polished hover/selection states"
  - "In-app file selection dialogues (open vault, import, transfer) feature path breadcrumbs, bookmark shortcuts (Home, Documents, Downloads), clean list/grid views, and clear primary CTA actions"
  - "File actions (new file, new folder, rename, move, delete) are accessible via contextual popovers with keyboard navigation and non-destructive confirmations"
---

## Context
The current file navigator (`Screen::Folders`) and in-app file dialogues (`Modal::Vault`, `Modal::Import`, `Modal::Transfer`) share the hostile design critiqued in the profile and vault switchers.
- **Outdated, Hostile UI:** In the Folders tree view, folder expand/collapse icons, raw text labels, and action buttons (`Add`, `Delete`, `Rename`) are presented with cramped hitboxes and unrefined alignment, feeling reminiscent of legacy desktop tree widgets rather than modern study environments like Obsidian or VS Code.
- **Dialogue UX Friction:** When opening or importing files in `Modal::Vault` or `Modal::Transfer`, users are presented with raw path string fields and utilitarian buttons ("Choose folder…", "Open vault") with minimal path validation guidance, no quick bookmark navigation, and no preview of selected contents.

## Description
Deliver a comprehensive visual and interaction overhaul of the file navigator and in-app file dialogues in `crates/studyfrog-ui`:

1. **Modern Folders Tree Explorer (`crates/studyfrog-ui/src/view/folders/`):**
   - **Visual Hierarchy:** Render subtle vertical indentation guidelines, chevrons for expandable folders, and distinct iconography for Markdown notes (`📄`), PDF documents (`📑`), flashcards (`🃏`), questions (`❓`), images (`🖼`), and stack directories (`📁`).
   - **Inline Search & Filter:** Add a fast search filter field at the top of the Folders toolbar that filters the tree in real time while preserving ancestor hierarchy.
   - **Contextual Operations:** Replace cramped inline buttons with a modern contextual action menu (or right-click popover) for file/folder creation, renaming, moving, and safe deletion.
   - **Breadcrumb Path Bar:** Display an interactive breadcrumb trail at the top of the folders view showing the active folder location with quick jump navigation.
2. **Contemporary In-App File Dialogue (`crates/studyfrog-ui/src/view/modal.rs`, `modal_vault.rs`, `modal_transfer.rs`):**
   - **Visual Navigation:** Modernize the file picker and vault selector modals with:
     - Left-hand sidebar containing quick bookmarks: User Home, Documents, Downloads, and Recent Vaults.
     - Breadcrumb bar with back/forward history buttons.
     - Search filter bar.
     - Clean list view showing file name, modification date, and file size.
   - **Clear Affirmative Actions:** Replace ambiguous buttons with high-salience primary CTAs ("Open Vault", "Import Cards") and clear cancel buttons.

## Acceptance Criteria
- [x] The file navigator (Folders screen) provides a modern tree explorer with indent guidelines, file-type icons (Markdown, PDF, image, stack), search filtering, and polished hover/selection states
- [x] In-app file selection dialogues (open vault, import, transfer) feature path breadcrumbs, bookmark shortcuts (Home, Documents, Downloads), clean list/grid views, and clear primary CTA actions
- [x] File actions (new file, new folder, rename, move, delete) are accessible via contextual popovers with keyboard navigation and non-destructive confirmations

## Design Notes
Touchpoints across crates:
- `crates/studyfrog-ui`: `view/folders/`, `view/modal.rs`, `view/modal_vault.rs`, `view/modal_transfer.rs`, `view/modal_import.rs`.
- Guardrails: Strictly adhere to P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-043 (Navigation, profile, and file management UX overhaul).
- EPIC-022 (Folders view and Obsidian-like file explorer).

## Open Questions
- None.
