# StudyFrog 0.3.0

Easier to find your way around, and StudyFrog now tells you what to study next. The full list is in `CHANGELOG.md`
under 0.3.0.

## What is new
- **Library like a list view.** Every item in one sortable table with chosen columns, a Standard search and a Query mode
  with an SQL-like language (`status = Learning AND familiarity < 0.4 ORDER BY due`), saved filters, and bulk changes to
  status, priority, difficulty, stack and tags with undo. Cards, notes and questions are made in a Create dialog laid
  out like Jira's, with Markdown editors that grow by dragging.
- **Stack pages.** A stack opens as an overview with tabs: Summary, List, Backlog, Roadmap, Board, Calendar, Reports,
  Sub-stacks and Goals. The old stack editing page stays as the item page.
- **Statuses.** Every item and stack has a status (derived from the history unless you set one); finished items are
  not planned.
- **Familiarity.** A value from 0.0 to 1.0 per item from views, ratings, reading speed and time without contact. It is
  shown in the reports, on the item page and in a Needs attention card on Home, steers which due items come first, and
  tempers the scheduler's interval (a relative factor on FSRS, on from the start). Settings can replay your own history
  to judge whether it helps.
- **Calendar planning.** The reviews the schedule plans show on the calendar per stack and day and can be dragged to
  another day, with a warning past an exam or due date and undo.
- **Rehearsal pacing.** A run can count down (it ends by itself at zero) or count up; the review and the Runs tab of the
  reports show the mode and the time used.
- **Import and export.** The library or a stack as JSON (with the review history in the advanced mode), CSV or a
  Markdown zip; imports with a preview and duplicate handling; Anki `.apkg` (with its review log) and Obsidian vaults.
- **Help inside the app.** `F1`, the `?` in the top bar or the Help button open the page of the screen you are on: 26
  pages (why StudyFrog, getting started, privacy, a how-to per screen, FAQ, glossary, shortcuts, the query language),
  a search, your own pages in the vault's `wiki/` folder, which the assistant can propose new pages and edits for, and
  a "First steps" tutorial offered once and repeatable.
- **Smaller things.** A notification center whose entries survive a restart; the vault's name in the top bar; settings
  in tabs by domain; teacher and subject names with commas.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.3.0-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.3.0-x86_64.AppImage
./StudyFrog-0.3.0-x86_64.AppImage
```
Put the file anywhere you like, for example `~/Applications/`, and start it from there; to get a menu entry, a
tool such as AppImageLauncher can add it. If it does not start because FUSE is missing, run it without FUSE:
`./StudyFrog-0.3.0-x86_64.AppImage --appimage-extract-and-run`.

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.3.0-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.3.0-x86_64.tar.gz
cd StudyFrog-0.3.0
./bin/studyfrog                                        # run it from here, or install it as below
```
To install it for your user (no root needed; make sure `~/.local/bin` is in your `PATH`):
```sh
install -Dm755 bin/studyfrog ~/.local/bin/studyfrog
install -Dm644 share/applications/studyfrog.desktop ~/.local/share/applications/studyfrog.desktop
install -Dm644 share/icons/studyfrog.png ~/.local/share/icons/studyfrog.png
```
For all users, copy the same three files under `/usr/local/` (`bin/`, `share/applications/`, `share/icons/`) with
`sudo`. To remove StudyFrog, delete those files; your vault is an ordinary folder and stays where it is.

### First start
Choose a folder for your vault when asked (an existing folder of Markdown works too); StudyFrog offers a short
"First steps" tutorial once, and `F1` opens help on every screen. Settings are kept in `~/.config/studyfrog/`.

## Not included
- **The diary** (self-reported progress set against measured results) moved to 0.4.0 at the owner's request.
- OCR for scanned PDFs, as in 0.2.0.

## Upgrading
The index database is migrated on first open (schema versions 16 and 17). New files in the vault: `filters.toml`
(saved filters), the `wiki/` folder (your help pages; Markdown there is no longer indexed as items, so move any stack
that lived in a top-level folder called `wiki`). New keys in `app.toml`: `library_columns`, `calendar_reviews`,
`stack_tabs`, `tutorial_offered`.

## What was checked how
- **By tests (headless):** the whole gate (`scripts/check.sh`: format, clippy, all tests, file size, layering, no-OOP,
  issues, board) passed on the release commit. The new screens, including help, the tutorial card, the library table,
  the stack tabs and the calendar drag and drop, are tested with `egui_kittest`, including accessible names.
- **Measured:** 10 000 library rows draw in about 11 ms per frame and filter in a few milliseconds; the benchmarks of
  this release are in `docs/contributing/BENCHMARKING.md`. On the release machine, which is about 2x slower than the one
  the baselines come from, the graph layout step took 8.9 ms against its 8 ms budget; the graph code is unchanged since
  0.1.0, so this is the machine, but it was not re-measured elsewhere.
- **Not run for this release:** the release-only tests of the language model (it is not installed on the release
  machine), and the AppImage was not started on a desktop session by hand (see `STATUS.md`).
- **Not looked at in a real window:** the help screen, the tutorial card, the stack tabs and the library table were
  tested headless only. Calendar drag and drop with a real mouse and IME input are unverified.
