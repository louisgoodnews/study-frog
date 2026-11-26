---
id: STORY-0243
type: story
title: "Horizontal scrollability and responsive sizing for constrained widgets across views"
status: backlog
epic: EPIC-073
parents: ['EPIC-073']
estimate: 3
labels: [m14, ui]
acceptance_criteria:
  - 'Shared helper(s) in studyfrog-ui provide opt-in horizontal scroll containers (ScrollArea::horizontal) with unobtrusive scrollbars and mouse-wheel/drag support for widgets that can exceed narrow panel widths'
  - 'Constrained single-line text inputs (such as Item Title, file/path bars, Tags, and Color hex inputs) expand to available width up to a sensible maximum and scroll horizontally without clipping text'
  - 'Wide or overflow-prone horizontal widget strips (such as Markdown tables, wide code blocks, long breadcrumb/path headers, and dense tab/chip bars in narrow panes) scroll horizontally cleanly instead of clipping or pushing parent panels past window edges'
  - 'Headless UI tests verify horizontal scrollability and zero unintended clipping at 960x640 and 1280x800'
---

## Context
Across several StudyFrog screens—most visibly the Item view (`BUG-0059`), Details side pane, long breadcrumb/path headers, and dense toolbars or Markdown code/table blocks—widgets placed inside fixed or narrow columns silently clip text or controls when content exceeds the column width. Providing clean, opt-in horizontal scrollability and responsive width sizing prevents content loss without breaking outer page layouts.

## Description
Introduce reusable horizontal scrollability and responsive-width primitives in `studyfrog-ui` (`crates/studyfrog-ui/src/view/kit.rs` or a dedicated helper module) and apply them to overflow-prone widgets:
1. **Horizontal Scroll Wrapper (`kit::hscroll` / `ScrollArea::horizontal`)**:
   - Bounded to the parent's `available_width()`, showing a subtle horizontal scrollbar only when content overflows so narrow panes stay intact without pushing sibling columns.
2. **Responsive Single-Line Inputs**:
   - Ensure single-line inputs (Item `Title`, `Tags`, `#RRGGBB` color code, path inputs) size responsively to available width with horizontal cursor scrolling enabled rather than truncating at tiny fixed pixel widths.
3. **Wide Content & Control Strips**:
   - Apply horizontal scrollability to wide Markdown tables, unwrappable code blocks, long vault-relative path strips in the Item header, and narrow tab/toolbar rows where wrapping is undesirable.

## Acceptance Criteria
- [ ] Shared helper(s) in `studyfrog-ui` provide opt-in horizontal scroll containers (`ScrollArea::horizontal`) with unobtrusive scrollbars and mouse-wheel/drag support for widgets that can exceed narrow panel widths
- [ ] Constrained single-line text inputs (such as Item Title, file/path bars, Tags, and Color hex inputs) expand to available width up to a sensible maximum and scroll horizontally without clipping text
- [ ] Wide or overflow-prone horizontal widget strips (such as Markdown tables, wide code blocks, long breadcrumb/path headers, and dense tab/chip bars in narrow panes) scroll horizontally cleanly instead of clipping or pushing parent panels past window edges
- [ ] Headless UI tests verify horizontal scrollability and zero unintended clipping at 960x640 and 1280x800

## Design Notes
- Keep helpers pure and modular under `crates/studyfrog-ui/src/view/`, adhering to Invariant P1 (zero inherent `impl` blocks) and Invariant P6 ($\le 250$ LOC per file).
- Coordinate with `BUG-0059` (Item view cut-offs) and `BUG-0058` (un-overhauled views/modals) so both immediate cut-offs and systemic horizontal overflow are resolved consistently.

## Dependencies
- `EPIC-073`, `BUG-0059`

## Open Questions
- Which narrow strips should prefer wrapping (`horizontal_wrapped`) vs horizontal scrolling (`ScrollArea::horizontal`) — e.g. chip groups wrap, while single-line paths, code blocks, and wide tables scroll horizontally.
