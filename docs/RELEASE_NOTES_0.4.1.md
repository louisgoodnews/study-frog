# StudyFrog 0.4.1

Notebook tabs in the item details view, inline stack due-date editing, Create dialog due dates and priority for all items, automatic full-window expansion for new notes, a simplified Jira-inspired Vaults switcher, and seamless CSV/TSV file importing. The full list is in `CHANGELOG.md` under `[0.4.1]`.

## What is new
- **Details View Notebook Overhaul & Stack Due Date Editing (`STORY-0093`).** The item details sidebar is restructured into a clean four-tab notebook (`Properties`, `Schedule`, `Relations`, `Learning`) so metadata, scheduling controls, wikilink relations, and familiarity drivers are never crowded together. Stacks now also expose direct inline due-date editing (`YYYY-MM-DD`, month-grid date picker, `Save` and `Clear`) in the `Schedule` tab, automatically syncing `EventKind::Deadline` entries on the vault calendar.
- **Create Dialog Due Date & Priority Across All Item Kinds (`STORY-0094`).** The Create dialog now exposes the `Priority` selector for stacks (`Lowest` to `Highest`) and the `Due date` field + reminder toggles (`15m`, `1h`, `1d`) across all creatable item types (`Stack`, `Card`, `Note`, `Question`), automatically creating calendar deadline sessions when a due date is set.
- **Full-Window Note Creation Modal & Default Write View (`STORY-0095`).** Selecting `Note` in the Create dialog automatically expands the modal to full window width (`widest(ctx)`) and defaults the Markdown body editor to `Write` mode, while preserving manual drag-to-resize, the `Shrink`/`Enlarge` toggle button, and item-type switching.
- **Simplified Vault Switcher Menu (`STORY-0099`).** Inspired by the Jira switcher, the top-bar `Vaults` menu now displays clean vault folder basenames in borderless, full-width selectable rows instead of raw filesystem paths. The open vault sits at the top with its parent directory as a muted subtitle; duplicate folder names across different locations are automatically disambiguated (`name · ancestor`), and hovering any row reveals the full path.
- **CSV & TSV Card Importer Auto-Detection (`BUG-0011`).** The card file importer now automatically detects and parses both comma-separated (`.csv`, RFC 4180 quoted fields, escaped quotes, embedded newlines, and 6-column AI prompt outputs `stack,kind,title,front,back,tags`) and tab-separated (`.tsv`) files without requiring manual conversion.

## Verification
- **Automated quality gates:** All 10 repository checks in `scripts/check.sh` (`cargo fmt --check`, `repo-lint`, `cargo clippy -D warnings`, workspace unit/integration/UI tests, doc-tests, 250-line file limit, dependency direction, zero inherent `impl` blocks, issue front-matter lint, and `BOARD.md` sync) pass on the release commit.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.4.1-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.4.1-x86_64.AppImage
./StudyFrog-0.4.1-x86_64.AppImage
```

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.4.1-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.4.1-x86_64.tar.gz
cd StudyFrog-0.4.1
./bin/studyfrog
```

### Upgrading from 0.4.0
Nothing to migrate: open your vault as before.
