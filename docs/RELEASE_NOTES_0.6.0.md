# StudyFrog 0.6.0

Exam mode, item keys and colours, learning analytics, cloze deletions, image occlusion, a focus timer, Python scripting with a control protocol, the rest of the Model Context Protocol work, a calmer and better-measured look, and a sharper, faster PDF viewer. This release completes the phase M12 scope that 0.5.0 began. The full list is in `CHANGELOG.md` under `[0.6.0]`.

## What is new

### Study
- **Exam mode.** **Exam** next to Flip cards and Type answers in the rehearsal setup: every card and question once, answered in writing, no skipping, no going back, no answer shown, no rating. A submitted answer is locked. Each answer is scored with the same comparison as typed answers, and the exam ends with a percentage, a grade on the combined scale (`A+ / 1+` for 95 % and above down to a pass at 60 %, `D / 4.0`), the per-item scores and a review. Results show in Reports > Runs and can fill the diary. Exams never change the review schedule (`docs/specs/exam-mode.md`).
- **Cloze deletions.** `{{c1::answer}}` and `{{c1::answer::hint}}` in a card or note; a run hides one group at a time and the groups take turns. A toolbar button and Ctrl+Shift+C write them.
- **Image occlusion.** Masks over a picture (also a figure saved from the PDF viewer), stored as an `occlusion` block; "hide all, guess one" or "hide one, guess one".
- **Cards from notes.** "Extract / Sync N card(s)…" on a note turns `Term :: Definition` and `Q:`/`A:` lines into cards and questions, linked back to the note; syncing again changes nothing unless the note changed.
- **Contrastive drill.** Two to four confused items side by side, with the shuffled answers to match; the distinction is kept as a hint or a note.
- **Focus timer.** A timer in the top bar (25/5, 50/10, 90/15 or your own) that keeps running on every screen; finished blocks are recorded against the calendar and offer a diary entry.

### Know your learning
- **Reports > Habits:** recall and speed by hour and weekday, and where fatigue sets in within a run, with a quiet line during a run when an answer takes far longer than usual.
- **Reports > Leeches:** items that keep failing and look-alike pairs, with edit, pause and drill.
- **Reports > Workload:** simulated reviews or study minutes for 30, 90 or 365 days with a typical value and a range, an exam pacer per stack, and the workload against the desired retention with a one-click "Apply recommended retention".
- **True retention** and forgetting curves by stability, next to what the scheduler expected.
- **Reports as CSV:** one table with the totals, retention, answer times, the forecast, the familiarity distribution, the run log and the per-stack numbers; "Export as CSV" writes to `exports/` in the vault, "Save CSV to…" to a folder you choose.

### Organise
- **Item keys** (`STACK_1`, `FLASHCARD_12`, `NOTE_3`, `QUESTION_7`): new items get one; "Assign keys…" gives older items theirs after a confirmation and can be undone. The query language has `key`, and typing an exact key into the search finds the item. A key badge with "Copy key" is on the item page.
- **Item colours:** a `color` in the front matter, chosen in Details > Properties; it tints the item page, its Details, the rehearsal card and the Library row, keeping text readable in both themes.

### Automate and connect
- **Control protocol.** `studyfrog --control-stdio` runs headless and answers one JSON line per request; every window and headless instance serves `run/control.sock`, and `studyfrog ctl <command>` talks to it: state, navigation, items, stacks, queries, rehearsal and exam runs, import and export, screenshots, logs and lifecycle events.
- **Python and scripts.** `python/studyfrog/` is a standard-library Python client of that protocol (no Python is linked into StudyFrog; ADR 0013). Scripts run as jobs in `<app home>/scripts/scripts.toml`, on a cron schedule or on events (`vault_opened`, `item_created`, `rehearsal_finished` and more), each with a timeout and its own read, write or destructive access. `studyfrog script run FILE` runs one by hand, and the **Console** in the sidebar takes commands and shows script output. Examples are in `examples/scripts/`.
- **MCP server.** Besides what 0.5.0 had: reports, diary, per-stack schedules, the due queue, analytics, the workload forecast and PDF pages; staged authoring tools (proposals wait for you in the Assistant's "Work the vault" tab unless the server runs with `--mcp-allow-write`), the standard prompts as MCP prompts, undoable structural tools behind `--mcp-allow-destructive`, and a localhost-only HTTP transport with a bearer token and origin checks (`--mcp-http 127.0.0.1:<port>`).
- **MCP client.** The Assistant can use the tools of external MCP servers (Connections > Tool servers) with your approval for every call that is not a trusted read-only one.
- **Connections:** "Test connection" for every provider.
- **Logs:** Settings > Data > "Open logs folder"; the log also records failed background work and import and export outcomes.

### Look and feel
- **Perceptual colour.** The palettes are OKLCH values checked by an APCA contrast test for every text and background pair in all 24 themes.
- **Type and spacing.** A four-size type scale, prose at about 70 characters a line, steady-width numbers, tonal cards instead of outlines, and one accent-filled button per view.
- **Zen mode.** Ctrl+Shift+F, F11 or the top-bar button hides the sidebar and status bar; Settings > Appearance can turn it on during runs.

### PDF viewer
- Pages stay sharp at every zoom and fit mode (and 400 % on a high-density display, which drew nothing before, now draws at the largest size the graphics card allows); the picture caches drop the least recently used picture; frame time and thumbnail requests are measured by tests.

### Changes you will notice
- The accent-filled button of the top bar ("+ Create"), Home ("Write entry") and "Add tool server" is now a quiet button with an accent label; the light theme has a light-grey page with white cards; the answer in a run is normal body text; the sidebar scrolls on a low window; on a narrow window the vault name in the top bar is cut short.

## Open points
- **STORY-0114 (typography) is in review:** its criterion asks for a reading measure of 620 to 680 px; in StudyFrog's font that is about 100 characters, so the measure is set to about 70 characters (about 450 points) instead. Please accept the character-based width or ask for the pixel one.
- **STORY-0124 (control protocol) is in review:** calendar, board, diary, reports, achievements, settings, assistant, PDF and folders can be reached with `nav`, but their actions are not commands; there is no widget tree; destructive stack operations such as deleting a stack are not offered. EPIC-036 and EPIC-038 are in review because of these two.
- Image occlusion has one schedule per card, not per mask, and no polygon masks. Exams do not feed familiarity and the CSV export has no exam columns.
- Scripting and the control socket need Unix. Deleting or moving an item raises no event.
- The new providers, the MCP server and client and the HTTP transport were tested against fixed requests and fake local servers, never against the real services or a real AI client.

## Verification
- **Automated quality gates:** all 10 repository checks in `scripts/check.sh` (`cargo fmt --check`, `repo-lint`, `cargo clippy -D warnings`, the workspace tests, the 250-line file limit, dependency direction, zero inherent `impl` blocks, issue front-matter lint, and `BOARD.md` sync) pass on the release commit.
- **Release-only tests:** with the language model installed, `cargo test --release -p studyfrog-candle` passes; comparing a 30-word answer took p50 24.4 ms, p95 24.7 ms (budget p95 < 150 ms).
- **Benchmarks:** every hot path is within its budget (see `docs/contributing/BENCHMARKING.md`, "Release 0.6.0 check"). The workload-against-retention simulation is the closest: 45.2 to 45.5 ms against 50 ms on a quiet machine, but 57.7 ms while the machine was busy.
- **Packaged binary on a real desktop:** the stripped release binary from the package opened a window, drew 120 frames and exited cleanly on Wayland and on X11, and rendered the Exam setup, Reports (Habits, Workload), Console and Prompts screens to PNG, which were looked at. Nobody has used the new screens by hand in an interactive window.
- **The AppImage wrapper** was not started for this release: it did not start in the environment used for the 0.5.0 check (the 0.1.0 image did not either), so the package was checked by its SHA-256 and by the binary inside it. The release workflow builds the published image.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.6.0-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.6.0-x86_64.AppImage
./StudyFrog-0.6.0-x86_64.AppImage
```

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.6.0-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.6.0-x86_64.tar.gz
cd StudyFrog-0.6.0
./bin/studyfrog
```

### Upgrading from 0.5.0
Open your vault as before. On the first start the vault's index is upgraded (three new versions: item keys and colours, exam results, focus blocks); the index can always be rebuilt from your files. Your files are not rewritten: an item gets a key only when it is created or when you choose "Assign keys…". Folders such as `scripts/` and `run/` appear in the application home (`~/.local/share/studyfrog`) when they are used.
