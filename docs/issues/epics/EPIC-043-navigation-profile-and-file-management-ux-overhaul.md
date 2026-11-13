---
id: EPIC-043
type: epic
title: "Navigation, profile, and file management UX overhaul"
status: done
parents: []
labels: [m12, ui, ux]
acceptance_criteria:
  - "Profile and vault switchers are overhauled with modern visual hierarchy, clear active status indicators, and thoughtful interaction flows"
  - "The file navigator (Folders screen) and in-app file dialogues feature polished tree hierarchies, path breadcrumbs, search filtering, and contemporary aesthetics"
  - "Rehearsal and exam modes provide an optional expandable drawer or slide-out panel allowing learners to view and add comments for the active card"
  - "All revamped menus and dialogues conform to the 4-tier modular type scale, OKLCH surface elevations, and accessible contrast guidelines"
---

## Context
User feedback indicates that the current profile switcher, vault switcher, and file navigator / in-app file dialogue suffer from a dated and hostile UI/UX:
- **Profile & Vault Switchers:** The existing switcher menus present raw full filesystem paths, unstyled selectable buttons, cramped popup geometries, and ambiguous destructive actions (e.g. "Really delete"). It is not immediately clear what action needs to be taken, and visual presentation lacks polish, whitespace, and intentional hierarchy.
- **File Navigator & File Dialogue:** The Folders view and in-app file picker modals (`Modal::Vault`, `Modal::Import`, `Modal::Transfer`) feel crude and utilitarian, lacking visual file-type distinction, breadcrumbs, search-as-you-type filtering, and modern folder navigation ergonomics.
- **In-Rehearsal Item Annotations:** During rehearsal runs and exams, learners need non-intrusive access to item comments and reflections through an expandable drawer without leaving active recall or opening the full editor notebook.

## Description
Conduct a comprehensive user experience and visual design overhaul of StudyFrog's navigation, profile management, and file dialogue systems across three core stories:

1. **Profile & Vault Switcher Overhaul (`STORY-0138`):**
   - Redesign the vault switcher popover into a structured panel with active vault header, recent vault list with last-opened timestamps and parent folder disambiguation, and prominent "Open Folder" / "Create New Vault" actions.
   - Redesign the profile menu with user avatar/initials badges, active profile status cards, theme toggles, and safe profile creation/switching/deletion dialogs.
2. **File Navigator & In-App File Dialogue Overhaul (`STORY-0139`):**
   - Modernize the Folders tree view with clean indent guides, file-type icons (Markdown, PDF, image, folder), contextual right-click/popover actions, and inline search.
   - Overhaul in-app file dialogues with breadcrumb bars, quick bookmarks (Home, Documents, Downloads, recent vaults), file grid/list toggle, and clear primary CTA buttons.
3. **Expandable Comments Drawer in Rehearsal & Exam Modes (`STORY-0140`):**
   - Provide an optional expandable drawer in rehearsal and exam modes to view, search, and add timestamped comments to the active card without interrupting rehearsal flow.

## Acceptance Criteria
- [x] Profile and vault switchers are overhauled with modern visual hierarchy, clear active status indicators, and thoughtful interaction flows
- [x] The file navigator (Folders screen) and in-app file dialogues feature polished tree hierarchies, path breadcrumbs, search filtering, and contemporary aesthetics
- [x] Rehearsal and exam modes provide an optional expandable drawer or slide-out panel allowing learners to view and add comments for the active card
- [x] All revamped menus and dialogues conform to the 4-tier modular type scale, OKLCH surface elevations, and accessible contrast guidelines

## Design Notes
Child stories:
- STORY-0138: Overhaul of profile and vault switchers (estimate: 5)
- STORY-0139: Overhaul of file navigator and in-app file dialogue (estimate: 5)
- STORY-0140: Expandable comments drawer in rehearsal and exam modes (estimate: 3)

Touchpoints across crates:
- `crates/studyfrog-ui`: `view/menus.rs`, `view/profiles.rs`, `view/modal.rs`, `view/folders/`, `view/run.rs`, `view/run_card.rs`, `theme.rs`, `typo.rs`.
- Guardrails: Strict compliance with P1 (no inherent `impl` blocks) and P2 (maximum 250 lines per Rust file).

## Dependencies
- EPIC-027 (Appearance and start-up settings).
- EPIC-036 (Evidence-based UI aesthetics, perceptual color science, and cognitive-load UX refinement).
- EPIC-039 (Universal item linking between stacks, cards, questions, answers, and notes).

## Open Questions
- None.
