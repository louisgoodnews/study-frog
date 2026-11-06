# StudyFrog 0.4.3

Unquoted multiline CSV field import support, moving stacks between parent stacks and standalone (top-level) after creation, and direct item deletion from the item viewer/editor with inline confirmation and undo support. The full list is in `CHANGELOG.md` under `[0.4.3]`.

## What is new
- **Unquoted Multiline CSV Import Recovery (`BUG-0014`).**
  - Both CSV importers (`studyfrog-ext-example` and `studyfrog-service::xfer_csv_map`) now detect and merge unquoted multiline continuation rows in headed CSV files (such as AI-generated `stack,kind,title,front,back,tags` exports containing unquoted line breaks or numbered lists inside `front` or `back`) before splitting columns, eliminating `line N: a card needs at least a front and a back` import failures.
- **Move Stack Between Parent Stack and Standalone (`STORY-0100`).**
  - Added `"Move stack…"` under `"More actions"` on the stack page and a `"Parent stack"` selector in the stack's `Properties` details tab.
  - Child stacks can be promoted to standalone (`top-level`) stacks and standalone or nested stacks can be moved under any valid parent stack while preserving all items' primary stack associations and spaced-repetition schedules.
- **Delete Item from Item Viewer & Editor (`STORY-0101`).**
  - Added `"Delete item…"` to `"More actions"` in the item editor toolbar (`Screen::Editor`) with an inline confirmation prompt (`"Delete this item?"` -> `"Yes, delete"` / `"Cancel"`).
  - Deleting an item removes its file from the vault, returns to the library view, and records an undo snapshot so `"Undo last change"` can restore it immediately.

## Verification
- **Automated quality gates:** All 10 repository checks in `scripts/check.sh` (`cargo fmt --check`, `repo-lint`, `cargo clippy -D warnings`, workspace unit/integration/UI tests, doc-tests, 250-line file limit, dependency direction, zero inherent `impl` blocks, issue front-matter lint, and `BOARD.md` sync) pass on the release commit.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.4.3-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.4.3-x86_64.AppImage
./StudyFrog-0.4.3-x86_64.AppImage
```

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.4.3-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.4.3-x86_64.tar.gz
cd StudyFrog-0.4.3
./bin/studyfrog
```

### Upgrading from 0.4.2
Nothing to migrate: open your vault as before.
