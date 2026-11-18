---
id: EPIC-057
type: epic
title: "UI/UX Streamlining, Universal Import Hub, and Cognitive De-Cluttering"
status: backlog
parents: []
labels: [m14, ui, ux]
acceptance_criteria:
  - 'Unified single Import action replaces fragmented card, stack, note, and question buttons'
  - 'Systematic contextual tooltips and hover hints across all navigation icons, buttons, and tools'
  - 'De-cluttered menu bars with progressive disclosure, reducing cognitive overload for new learners'
  - 'Lightweight, modern visual rhythm maintaining deep power features under intuitive secondary disclosures'
---

## Context
StudyFrog has accumulated numerous specialized capabilities across 22 crates, but interface density has outpaced intuitive usability. Overpopulated menus, redundant buttons (e.g. separate card import vs stack/note import), and a lack of hover hints make the application feel overly complicated rather than light and supportive.

## Description
Conduct a comprehensive user experience refinement focused on simplicity, clarity, and delight:
1. **Unified Universal Import**: Replace fragmented, disparate import buttons with a single clear "Import" action featuring intelligent content auto-detection.
2. **Contextual Tooltips & Guidance**: Add helpful hover tooltips across the navigation rail, action icons, headers, and modal actions, including keyboard shortcut badges.
3. **Menu & Action De-Cluttering**: Clean up crowded headers and menus by grouping secondary tools into progressive disclosures (`...` overflow menus) and increasing breathing room.
4. **Lightweight Persona Alignment**: Rebalance information hierarchy so the app feels approachable and focused for daily study while keeping advanced features effortlessly discoverable.

## Acceptance Criteria
- [ ] Unified single Import action replaces fragmented card, stack, note, and question buttons
- [ ] Systematic contextual tooltips and hover hints across all navigation icons, buttons, and tools
- [ ] De-cluttered menu bars with progressive disclosure, reducing cognitive overload for new learners
- [ ] Lightweight, modern visual rhythm maintaining deep power features under intuitive secondary disclosures

## Design Notes
Implement tooltips using `egui::Response::on_hover_text` and `on_hover_ui`. Strictly use palette color tokens from `palette_of(ui)` (no raw RGB). Follow the established rule of visual halves for spacing and padding.

## Dependencies
None.

## Open Questions
None.
