---
id: BUG-0058
type: bug
title: "Several views and modals are missing the EPIC-070/EPIC-073 redesign overhaul"
status: backlog
estimate: 3
labels: [m14, ui]
acceptance_criteria:
  - "The Items ('All items' / stack list) view uses the redesign style: gradient headline panel, redesign card framing for the filter bar and item table, and consistent rail highlight styling for the left Stacks sub-rail"
  - "The 'Create an item' modal uses the redesign card/surface tokens, clean header styling, structured field cards, and consistent spacing without cramped toolbar clusters"
  - "The 'Import and export' modal uses the redesign header/card hierarchy for its mode tabs, 'Found on this computer' discovered sources list, and file/folder picker controls"
  - "The Profile / Workspace popover menu uses the redesign card/surface tokens and consistent section grouping across Dark, Light, and High-contrast themes"
  - "The Vault switcher popover/view uses the redesign card/surface tokens, active vault indicator, and clean section grouping"
  - "Headless UI and visual-rhythm tests pass across all updated views and modals with zero missing accessible names"
---

## Context
After completing the UI redesign (`EPIC-070`, `EPIC-073`), visual inspection of StudyFrog `1.2.0` revealed four views/overlays that were missed and still render in the pre-redesign flat style:
1. **Items (`All items`) screen** (`crates/studyfrog-ui/src/view/items.rs`, `library.rs`): renders a plain `"All items"` heading with no gradient headline panel, unframed filter bar and item table, and plain `STACKS` sub-rail styling.
2. **Create an item modal** (`crates/studyfrog-ui/src/view/modal_create.rs`): uses the old flat modal container, plain title row, and unframed dense editor/toolbar sections.
3. **Import and export modal** (`crates/studyfrog-ui/src/view/modal_transfer.rs`): uses the old flat dialog styling with plain `"Import and export"` header and unframed `"Found on this computer"` list and file picker controls.
4. **Profile / Workspace popover** (`crates/studyfrog-ui/src/view/profiles.rs`): uses the old flat popover layout with unframed sections (`Appearance`, active profile, `Switch profile`, `Add new profile`, `Close vault`).
5. **Vault switcher** (`crates/studyfrog-ui/src/view/vault_menu.rs`): still uses the pre-redesign popover/menu layout without the redesign card/surface hierarchy.

(Additional un-overhauled views discovered during collection can be appended to this bug.)

## Description
Bring the four listed views and modals into visual parity with the `EPIC-070` / `EPIC-073` redesign style:
- **Items (`All items`):** Add the gradient headline panel (`gradient_panel::panel`) showing the active selection title and item count, wrap the search/filter bar and item table in redesign cards (`home_hero::card` / `style_of(&palette_of(ui))`), and align the `STACKS` sub-rail with the redesign rail highlight tokens.
- **Create an item modal:** Apply redesign modal/card surface tokens, a crisp header bar, clean card grouping for prompt/front/back editors and metadata fields, and balanced spacing.
- **Import and export modal:** Apply redesign header/tab styling and card framing around discovered sources (`Found on this computer`) and the file/folder picker section.
- **Profile / Workspace popover:** Apply redesign card/surface tokens, avatar/badge hierarchy, and clean section grouping for Appearance, active profile, profile switcher, new profile creation, and Close vault.
- **Vault switcher:** Apply redesign card/surface tokens, active vault badge/highlight, and clean spacing for switching, opening, and managing vaults.

## Acceptance Criteria
- [ ] The Items (`All items` / stack list) view uses the redesign style: gradient headline panel, redesign card framing for the filter bar and item table, and consistent rail highlight styling for the left Stacks sub-rail
- [ ] The `Create an item` modal uses the redesign card/surface tokens, clean header styling, structured field cards, and consistent spacing without cramped toolbar clusters
- [ ] The `Import and export` modal uses the redesign header/card hierarchy for its mode tabs, `Found on this computer` discovered sources list, and file/folder picker controls
- [ ] The Profile / Workspace popover menu uses the redesign card/surface tokens and consistent section grouping across Dark, Light, and High-contrast themes
- [ ] The Vault switcher popover/view uses the redesign card/surface tokens, active vault indicator, and clean section grouping
- [ ] Headless UI and visual-rhythm tests pass across all updated views and modals with zero missing accessible names

## Design Notes
- Affected source files:
  - `crates/studyfrog-ui/src/view/items.rs` and `crates/studyfrog-ui/src/view/library.rs`
  - `crates/studyfrog-ui/src/view/modal_create.rs`
  - `crates/studyfrog-ui/src/view/modal_transfer.rs`
  - `crates/studyfrog-ui/src/view/profiles.rs`
  - `crates/studyfrog-ui/src/view/vault_menu.rs`
- Use only `palette_of(ui)` and `style_of(&palette_of(ui))` tokens (no raw RGB literals), keep all files $\le 250$ LOC (Invariant P6), preserve all existing accessible widget names and keyboard shortcuts, and check contrast in both Dark and Light themes.

## Dependencies
None.

## Open Questions
- Whether any additional views or dialogs should be added to this checklist as the user finishes collecting them.
