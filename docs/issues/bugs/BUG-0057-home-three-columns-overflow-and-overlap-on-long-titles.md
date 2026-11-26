---
id: BUG-0057
type: bug
title: "Home screen three-column cards overflow and overlap adjacent columns on long item titles"
status: done
estimate: 2
labels: [m14, ui]
acceptance_criteria:
  - "Each of the three Home columns ('Due now', 'Coming up', 'Recent activity') stays strictly within its 1/3 column width without overflowing into or under adjacent columns or past the window edge"
  - "Long item titles, due badges, and upcoming event rows wrap or truncate cleanly inside their column card (with full title available on hover when truncated)"
  - "All three column cards align cleanly in width, spacing, and vertical presentation at standard window sizes (e.g. 960x640, 1280x800, and wider)"
  - "Headless UI test covers long item titles in all three Home columns and verifies no column rect exceeds its allocated column bounds"
---

## Context
Reported by the user with a screenshot of the Home screen (`Psychologische Diagnostik` vault) showing the three-column section (**Due now**, **Coming up**, **Recent activity**):
- In **Due now**, long flashcard titles (e.g. *"Was ist die einzige gemeinsame Verursachung, wenn Item..."*, *"Was passiert mit der Iteminformationsfunktion I_iv..."*) do not wrap or truncate at the column edge. The **Due now** card expands far past its 1/3 width and bleeds horizontally underneath the **Coming up** card and into the **Recent activity** column.
- In **Recent activity**, long note and card titles (e.g. *"Probabilistische Testtheorien (PTT) sind wie ein intelligentes..."*, *"Mit welcher Art von Frage wird beim adaptiven Testen in der..."*) overflow and are clipped at the right edge of the card/window.
- The three column cards also have mismatched heights/boundaries where the overflowing left card visually collides with the middle and right cards.

## Description
Constrain each Home column card (**Due now**, **Coming up**, **Recent activity**) to its allocated column width so that long titles and rows wrap or ellipsize cleanly inside the card instead of expanding the card across adjacent columns or past the right edge of the window, and align the three columns cleanly.

## Acceptance Criteria
- [x] Each of the three Home columns (`Due now`, `Coming up`, `Recent activity`) stays strictly within its 1/3 column width without overflowing into or under adjacent columns or past the window edge
- [x] Long item titles, due badges, and upcoming event rows wrap or truncate cleanly inside their column card (with full title available on hover when truncated)
- [x] All three column cards align cleanly in width, spacing, and vertical presentation at standard window sizes (e.g. 960x640, 1280x800, and wider)
- [x] Headless UI test covers long item titles in all three Home columns and verifies no column rect exceeds its allocated column bounds

## Design Notes
- Root cause in `crates/studyfrog-ui/src/view/home.rs` and `crates/studyfrog-ui/src/view/home_cards.rs`:
  - `draw` (`home.rs:84`) lays out the three sections with `ui.columns(3, |c| ...)`, and `column_card` (`home.rs:94-99`) only sets `ui.set_width(ui.available_width())` (minimum width) without bounding `max_width` or clipping, and ignores `_fill`.
  - Item rows in `due_column` (`home.rs:118`), `ahead_column` (`home_cards.rs:43`), and `recent_column` (`home_cards.rs:65`) use `ui.horizontal(|ui| ...)` with `ui.link(&i.title)` and `ui.label(...)`, which do not wrap or truncate in `egui` and push the enclosing `Frame` beyond the column's width.
  - `stacks_waiting` (`home.rs:149`) uses an unbounded `egui::Grid` that can also push the card width if stack titles are long.
- Fix direction: bound each column card to its column width (`ui.set_max_width(col_width)` / `ui.set_width(col_width)`), use `horizontal_wrapped` or truncated link labels (`TextWrapMode::Truncate` / `Wrap`) with full-title hover tooltips for item rows, and align column card heights/spacing consistently.
- Guardrails: Invariant P6 (files $\le 250$ LOC), palette tokens via `palette_of(ui)`, accessible widget names preserved.

## Dependencies
None.

## Open Questions
- Closed with wrapping (`horizontal_wrapped` + `TextWrapMode::Wrap`) so full titles stay readable without overflowing adjacent columns.

## Fix note (2026-11-26)
- Bounded `column_card` in `crates/studyfrog-ui/src/view/home.rs` with `ui.set_min_size(egui::vec2(inner, fill))`, `ui.set_max_width(inner)`, and `TextWrapMode::Wrap`.
- Replaced unbounded `ui.horizontal` item rows in `due_column`, `ahead_column`, and `recent_column` (and the rigid 3-column `Grid` in `stacks_waiting`) with `ui.horizontal_wrapped`.
- Added headless bounds test `crates/studyfrog-ui/tests/home_columns.rs` verifying zero horizontal overflow at `1280x800` and `960x640` (`cargo test -p studyfrog-ui --test home --test home_columns`: 9/9 passed).
