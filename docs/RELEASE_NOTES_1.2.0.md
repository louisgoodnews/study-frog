# StudyFrog 1.2.0

A feature and polish release focused on the Folders file manager and Home screen layout.

## Added

- **Nautilus/Thunar-style Folders view (`EPIC-074`):**
  - Navigation bar with Back, Forward, and Up buttons, clickable breadcrumb path bar, inline folder filter, and a **Grid / List / Tree** view switcher.
  - **Icon Grid view** with responsive tiles, vector-drawn icons for folders, stack folders, Markdown notes, PDFs, and images, wrapped labels, child item counts, selection highlights, double-click open/enter, inline rename, context menus, and drag-and-drop moves.
  - **Detailed List view** with clickable sortable column headers (**Name**, **Type**, **Size / Items**, **Modified**), sort-direction indicators, and per-row star/bookmark toggles.
  - Expanded 8-way sorting (Name, Modified, Type, Size/Items — ascending or descending) with **Folders first** and **Starred** filter toggles, plus a bottom-right selection status pill.

## Fixed

- **Home screen three-column overflow (`BUG-0057`):** Bounded the `Due now`, `Coming up`, and `Recent activity` column cards to their column widths and wrapped long item titles and stack rows so cards no longer overlap adjacent columns or clip at the window edge at `960x640`, `1280x800`, or wider sizes.

## Install

Download `StudyFrog-1.2.0-x86_64.AppImage` (or `StudyFrog-1.2.0-x86_64.tar.gz`) from this release, verify it against the `.sha256` file printed next to it, and launch it from a desktop session.

## Upgrading from 1.1.1

Drop-in upgrade: existing vaults, settings, and databases open without manual migration.
