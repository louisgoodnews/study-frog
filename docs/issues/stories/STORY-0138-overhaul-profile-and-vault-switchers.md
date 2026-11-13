---
id: STORY-0138
type: story
title: "Overhaul of profile and vault switchers"
status: done
epic: EPIC-043
parents: [EPIC-043]
estimate: 5
labels: [m12, ui, ux]
acceptance_criteria:
  - "The vault switcher popover renders an elevated, thoughtful card layout with active vault indicators, disambiguated folder paths, last-opened metadata, and prominent actions to open or create vaults"
  - "The profile switcher features visual profile avatars/initials, clear active profile cards, non-hostile profile creation and deletion workflows, and unified theme controls"
  - "All switcher interactions feature accessible keyboard navigation, responsive hover states, and compliant APCA contrast ratios"
---

## Context
Currently, profile and vault switching leave the user with a hostile UI and UX. Specific deficiencies identified include:
1. **Hostile Visual Presentation:** Switcher menus are narrow, cramped popovers (`260-300 px`) using default selectable buttons with minimal internal padding and visual edge entropy.
2. **Ambiguous Actions & UX Friction:** In the vault switcher, full raw filesystem paths are either truncated or clumsily disambiguated; it is not immediately obvious how to switch to a recent vault versus opening a new directory. In the profile menu, destructive actions like "Really delete" appear inline next to the switch button with awkward "Keep" buttons, creating anxiety and poor affordance.
3. **Lack of Quality and Thoughtfulness:** The menus lack visual hierarchy (e.g. active badge, last accessed time, storage path breakdown, profile avatars) and fail to present StudyFrog as a polished, thoughtful learning environment.

## Description
Redesign and overhaul both the vault switcher and profile switcher menus in `crates/studyfrog-ui`:

1. **Modern Vault Switcher Popover (`crates/studyfrog-ui/src/view/menus.rs`, `navbar.rs`):**
   - Provide an expanded, spacious popover layout with distinct structural sections:
     - **Active Vault Card:** Display the current vault with an accent icon, large title, parent path breadcrumb, and a quick "Reveal in Folders" / "Vault Settings" shortcut.
     - **Recent Vaults List:** Render selectable vault cards showing folder name, subtle parent path, item count badge (when available), and relative last-opened timestamp.
     - **Action Footer:** Clearly distinct primary buttons: "Open Existing Folder…" (native folder picker) and "New Vault…" (guided creation dialog), eliminating ambiguous raw text boxes.
2. **Modern Profile Switcher & Management (`crates/studyfrog-ui/src/view/profiles.rs`, `menus.rs`):**
   - Redesign the profile menu with a contemporary account/profile switcher aesthetic:
     - **Profile Header:** Show the active profile name with a colored initials avatar badge, active status pill, and current theme preset preview.
     - **Profile List:** Clean card list of alternate profiles with smooth one-click switching and a clean overflow menu (`···`) for renaming or deleting non-default profiles.
     - **Dedicated Profile Dialog:** Move profile creation ("New Profile") and deletion into a dedicated, safe confirmation modal with clear explanations and cancel options instead of cramped inline buttons.
     - **Unified Theme Switcher:** Integrated segmented theme control (System / Dark / Light) with visual icons.

## Acceptance Criteria
- [x] The vault switcher popover renders an elevated, thoughtful card layout with active vault indicators, disambiguated folder paths, last-opened metadata, and prominent actions to open or create vaults
- [x] The profile switcher features visual profile avatars/initials, clear active profile cards, non-hostile profile creation and deletion workflows, and unified theme controls
- [x] All switcher interactions feature accessible keyboard navigation, responsive hover states, and compliant APCA contrast ratios

## Design Notes
Touchpoints across crates:
- `crates/studyfrog-ui`: `view/menus.rs`, `view/profiles.rs`, `view/navbar.rs`, `view/popup.rs`, `profile_ui.rs`, `vault_ui.rs`.
- Aesthetic alignment: Follow `docs/research/ui-aesthetics-and-ux-science.md` with OKLCH surface elevations, 8/16 px padding, and modular typography.
- Guardrails: Strictly adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-043 (Navigation, profile, and file management UX overhaul).
- EPIC-027 (Appearance and start-up settings).

## Open Questions
- None.
