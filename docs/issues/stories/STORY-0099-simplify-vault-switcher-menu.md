---
id: STORY-0099
type: story
title: "Simplify vault switcher menu"
status: done
epic: EPIC-031
parents: [EPIC-031]
estimate: 3
labels: [m11, ui]
acceptance_criteria:
  - "The vault switcher popup displays vault folder names instead of raw multi-line full filesystem paths"
  - "Recent vaults are rendered as clean, borderless selectable list rows with generous spacing and hover states inspired by the Jira switcher"
  - "Vaults sharing a parent directory are either grouped under a collapsible parent heading or show a subtle muted parent subtitle for disambiguation, with full paths shown on hover"
---

## Context
Currently, the Vaults switcher popup (`Menu::Vaults` in `crates/studyfrog-ui/src/view/menus.rs`) renders each recent vault as a standard bordered `ui.button` containing the entire raw filesystem path (e.g. `/home/user/Dokumente/Studium/WiSe 2026-27/Kognitive Neurowissenschaft`). Long paths wrap across two or three lines inside bulky button frames, repeating identical directory prefixes and creating visual clutter. Compared to modern switcher menus (such as the Jira app/workspace switcher), the current popup is hard to scan quickly.

## Description
Redesign and simplify the Vaults switcher popup (`crates/studyfrog-ui/src/view/menus.rs`) following the clean visual hierarchy of the Jira switcher:
1. Active Vault Header:
   - Display the currently open vault at the top with a distinct icon badge, bold vault title (e.g. `Multivariate Verfahren`), and a weak/muted subtitle showing its parent directory context (e.g. `WiSe 2026-27`).
2. Clean Borderless List Items:
   - Replace bulky multi-line bordered `ui.button(path)` widgets with full-width, borderless selectable rows (`egui::Button::selectable` or frameless hover rows with `min_size(vec2(width, 32.0))` and generous padding).
   - Display only the human-readable vault name (the folder basename, e.g. `Kognitive Neurowissenschaft`) as the primary label.
3. Disambiguation & Grouping:
   - When multiple vaults reside under a common parent directory (e.g. `WiSe 2026-27`) or when two vaults share the same basename (e.g. `Psychologische Diagnostik und Evaluation` under two different trees), group entries under a collapsible parent section with a chevron or render a compact muted subtitle under the vault name.
   - Attach `.on_hover_text(full_path)` to every entry so the full filesystem path remains inspectable on hover.
4. Footer Action:
   - Separate the bottom `"Open another vault…"` action with a clean divider and subtle icon row.

## Acceptance Criteria
- [x] The vault switcher popup displays vault folder names instead of raw multi-line full filesystem paths
- [x] Recent vaults are rendered as clean, borderless selectable list rows with generous spacing and hover states inspired by the Jira switcher
- [x] Vaults sharing a parent directory are either grouped under a collapsible parent heading or show a subtle muted parent subtitle for disambiguation, with full paths shown on hover

## Design Notes
- Crate touchpoints:
  - `crates/studyfrog-ui/src/view/menus.rs`: Refactor `vaults()` and increase `Menu::Vaults` popup width if needed (e.g. `280.0`–`300.0`) with single-line truncation or clean subtitle layout.
  - Extract a helper function to derive `(vault_name, parent_hint)` from a path string without exceeding the 250-line limit.
- Release target: version 0.5.0 (Phase M12).

## Dependencies
EPIC-032.

## Open Questions
None.
