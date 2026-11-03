# StudyFrog 0.3.1

Fixes and a calmer look, from the first days of using 0.3.0. The full list is in `CHANGELOG.md` under 0.3.1.

## What is new
- **StudyFrog's own title bar.** Drag the top bar to move the window (the window manager does the moving, so snapping
  works), double-click it to maximise, use the large minimise and maximise buttons and the red close button, and drag
  the window's edges to resize it. The red button is the Exit button: it asks first when the editor has unsaved changes.
- **A calmer layout.** More room around every screen; a reading column for the rehearsal setup, settings and help;
  grouped cards and quieter headings; the library's search on one row with a narrower stack list; a stack page with one
  main button (Start rehearsal) and a "More actions" menu, and a two-column Summary. Rows of a name and a bar (priorities,
  ratings, familiarity) are aligned so the bars start at one place and have one length. Scroll bars no longer sit on top
  of the content next to them.
- **A progress bar in the rehearsal**: "3 done · 5 left", counted against the items still open (an item whose answer
  shows is open until rated or skipped; items put back to repeat count as open and are named).
- **Safer Create dialog.** Cancel, the close button, Esc or a click outside ask "Discard what you wrote?" when something
  is written; the dialog's header no longer cuts off the type name.
- **Import suggestions.** The Import tab lists Obsidian vaults (from Obsidian's own list), Anki packages in Downloads,
  Documents and Desktop, and Anki profiles with the hint to export a package. Nothing is read until you choose one.
- **Developer:** `studyfrog --screenshot FILE --screen NAME` saves a PNG of a screen of the running app.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.3.1-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.3.1-x86_64.AppImage
./StudyFrog-0.3.1-x86_64.AppImage
```
Put the file anywhere you like, for example `~/Applications/`, and start it from there; to get a menu entry, a
tool such as AppImageLauncher can add it. If it does not start because FUSE is missing, run it without FUSE:
`./StudyFrog-0.3.1-x86_64.AppImage --appimage-extract-and-run`.

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.3.1-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.3.1-x86_64.tar.gz
cd StudyFrog-0.3.1
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

### Upgrading from 0.3.0
Nothing to migrate: no new files or settings keys. Open your vault as before.

## Not included
- The diary (planned for 0.4.0) and the calmer look for the screens not done yet (Calendar, Board, Reports,
  Achievements, Assistant, Graph, the Create dialog's optional fields, the sidebar): `docs/specs/ui-calm.md`.
- Reading Anki's own database: export a package from Anki instead (see Import suggestions above).

## What was checked how
- **By tests (headless):** the whole gate (`scripts/check.sh`) passed on the release commit, including new tests for
  the title bar buttons, the discard question, the import suggestions, the aligned bars, the scroll bar spacing and the
  progress bar's numbers.
- **In the running app:** the new layouts were looked at in screenshots of the real app (`--screenshot`), on X11, and
  the owner tried the title bar and the layout on a GNOME/Wayland machine and was satisfied.
- **Not run for this release:** the benchmarks and the release-only language-model tests (neither area changed), and
  the progress bar was seen at "0 done" only; a partly filled bar and the repeat wording are covered by tests only.
