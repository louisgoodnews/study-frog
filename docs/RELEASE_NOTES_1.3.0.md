# StudyFrog 1.3.0

A polish, bug-fix, and Match mode usability release.

## Added

- **Vault-sourced Match mode (`EPIC-075`):**
  - Loads real `Card` front/back and `Question` prompt/answer pairs from the open vault (with Markdown stripping and deduplication) instead of hardcoded biology demo cards.
  - Scope picker (**Entire vault** or any stack), prioritization filter (**All**, **Due first**, **Least familiar**), pair-count selector (**4**, **6**, **8** pairs), distinct **Restart** vs **New round** actions, centered responsive tile grid with `Prompt`/`Answer` badges and full-text hover tooltips, and an honest empty state when fewer than 2 valid pairs exist.
- **Rehearse setup polish (`STORY-0251`):**
  - Highlights the active preset in Step 2 (**How**) and adds quick-pick run item-limit chips (**No limit**, **10**, **20**, **30**, **50**) with inline validation feedback.
- **Horizontal scrollability (`STORY-0243`):**
  - Added shared `kit::hscroll` container for constrained horizontal strips such as long item paths.

## Fixed

- **Item view cut-offs (`BUG-0059`):** Full-width gradient headline, `Title` input, Card/Question `Front`/`Back` editors, and widened Details > Properties Color `#RRGGBB` input.
- **Redesign parity (`BUG-0058`):** Applied redesign gradient and card framing to Items (`All items`), Create an item modal, Import and export modal, Profile popover, and Vault switcher.
- **Duplicate button hover tooltips (`BUG-0060`):** Deduplicated stacked tooltips on Rehearse `Show answer` and Focus timer buttons.
- **Text wrapping & right-edge pills (`STORY-0250`):** Expanded Rehearse card and Markdown preview wrapping to full card width and prevented Diary rating pills and `kit::pill` chips from crushing vertically at the right edge.
- **Question & answer-type imports (`STORY-0249`):** Preserved `answer_type` (`open`, `single`, `multiple`) across JSON, TOML, RON, and CSV export/import round-trips and allowed choice questions without a separate reference answer.

## Install

Download `StudyFrog-1.3.0-x86_64.AppImage` (or `StudyFrog-1.3.0-x86_64.tar.gz`) from this release, verify it against the `.sha256` file printed next to it, and launch it from a desktop session.

## Upgrading from 1.2.0

Drop-in upgrade: existing vaults, settings, and databases open without manual migration.
