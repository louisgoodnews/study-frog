# StudyFrog 0.5.0

An application home with logs, a Model Context Protocol server for AI tools, Google Gemini and DeepSeek as assistant providers, four more prompts and a prompt screen, many more achievements with a per-stack view, a CSV export of the reports, and a fix for blurred PDF pages. The full list is in `CHANGELOG.md` under `[0.5.0]`.

**This release is part of the planned scope of phase M12.** About a third of the planned stories are built; several of those are partial. What is not built is listed under "Not in this release" and moves to the next version.

## What is new
- **Application files and logs (`STORY-0122`).**
  - StudyFrog keeps its own files in one home: `$STUDYFROG_HOME`, else `$XDG_DATA_HOME/studyfrog`, else `~/.local/share/studyfrog`, with `config/`, `models/` and `logs/`. `studyfrog --print-paths` lists them, and Settings > Data shows them with a Copy button.
  - Every notice is written to `logs/studyfrog.log` (text) and `logs/console.jsonl` (JSON Lines), rotated at 5 MB, three files each. `studyfrog logs [--tail N] [--follow] [--json]` shows them.
  - The assistant's call log moved to `logs/agent.log` in the same home.
- **A Model Context Protocol server (`STORY-0117`, `STORY-0118`, `STORY-0120`).**
  - `studyfrog --mcp [--vault PATH]` lets AI tools such as Claude Desktop, Cursor or Zed read your vault over standard input and output. It is read-only: the vault summary, each stack, each item as Markdown, and a search that uses the query language (plain words are searched in the text). A session changes nothing in the vault.
  - The Assistant's connections tab has **Copy MCP server config** with the settings block for the open vault.
  - Behind it is a pure JSON-RPC 2.0 router with plain-data tools, resources and prompts and a policy for tools that write; a new tool is one more entry in a list.
- **Google Gemini and DeepSeek (`STORY-0083`, `STORY-0084`).** Two more providers for the assistant, with their default addresses and suggested models as buttons in the connection form. The Gemini key goes in a header, never in the address; safety stops are reported with their reason.
- **Prompts (`STORY-0088` to `STORY-0092`, `TASK-0233`).**
  - Four new prompts: explain a file at three levels, be examined on a file, learn a file by Socratic dialogue, and map its concepts. The two chat prompts hold a normal conversation and give the importable file when you type `EXPORT`.
  - Prompts have categories and a search, and some have options (the depth of an explanation, the number of exam questions) that go into the text.
  - The prompts have a screen of their own, **Prompts**, in the sidebar.
- **Achievements (`STORY-0086`, `STORY-0087`).** 32 built-in tiers instead of 8: streaks from 3 to 365 days, runs, ratings and mastery, and four tiers of learned and mastered items for every stack. The screen has a filter (all, global, one stack) and groups by category and stack, and a stack's page shows how many of its achievements are unlocked. The ids of the first set are unchanged, so earlier unlocks stay.
- **Reports as CSV (`STORY-0098`).** **Export as CSV** on the Reports screen writes one table (`section,subject,metric,value`) with the review total, retention, answer times, the review forecast, the familiarity distribution and the run log to `exports/` in the vault.
- **PDF viewer (`BUG-0010`, `STORY-0085`).** Pages could stay blurred after the sidebar thumbnails had been drawn: a small picture shown while the sharp one was rendered took over the sharp one's texture. Textures are now kept by their pixel size. Full-size pages and thumbnails are also trimmed apart (6 pages, 60 thumbnails), so scrolling the thumbnails no longer pushes out the page being read and far fewer large textures stay in memory.

## Not in this release
These stories were planned for 0.5.0 and are not built; they move on:
- Item keys and colours (`STORY-0096`, `STORY-0097`, `STORY-0123`), the Exam run mode (`EPIC-033`), the learning analytics and forecasting (`STORY-0102` to `STORY-0107`), cloze deletions, image occlusion, note extraction, the contrastive drill and the Pomodoro timer (`STORY-0108` to `STORY-0112`), the colour, type and layout refinements (`STORY-0113` to `STORY-0116`), the Python modding API, and the interactive control protocol (`STORY-0124`).
- Partly built: the MCP server has no resources for reports or the diary, no tools for the due queue, analytics, forecast or PDF pages, no write tools (`STORY-0119`), no HTTP transport and no client host (`STORY-0121`); the CSV export has no per-stack statistics; the PDF viewer keeps no parsed document between renders (measured: parsing is about a tenth of a render) and evicts by age, not by last use; there is no "test connection" button for the new providers.

## Known limits
- The Prompts, Achievements, Reports, Assistant connections and stack-page screens were rendered by the packaged binary on a real desktop and looked at. Settings > Data ("Application files") and every click path were tested headless only.
- Gemini and DeepSeek were never run against the real services, only against fixed requests and answers. The MCP server was run as the real binary over pipes, not with a real AI client.
- The page fix for blurred PDFs is tested on the texture keys; whether pages are sharp at every zoom level and fit mode has not been seen in a window.

## Verification
- **Automated quality gates:** all 10 repository checks in `scripts/check.sh` (`cargo fmt --check`, `repo-lint`, `cargo clippy -D warnings`, the workspace tests, the 250-line file limit, dependency direction, zero inherent `impl` blocks, issue front-matter lint, and `BOARD.md` sync) pass on the release commit.
- **Release-only tests:** with the language model installed, `cargo test --release -p studyfrog-candle` passes; comparing a 30-word answer took p50 28.9 ms, p95 31.0 ms (budget p95 < 150 ms).
- **Benchmarks:** every hot path is within its budget (see `docs/contributing/BENCHMARKING.md`, "Release 0.5.0 check").
- **The packaged binary on a real desktop (Wayland, and X11):** the stripped release binary from the package opened a window, drew 120 frames and exited cleanly; it also rendered the screens named above to PNG.
- **The AppImage wrapper** did not start in the environment used for this check (the 0.1.0 AppImage did not start there either, so this is the environment, not a change), so the AppImage was checked by its SHA-256 and by the binary inside it. The release workflow builds the published image; starting it on a desktop where AppImages run is still to do.
- **The MCP server** was run as the real binary over pipes with a sample vault (initialize, a notification, a search, a resource read): only protocol lines on standard output, nothing on standard error.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.5.0-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.5.0-x86_64.AppImage
./StudyFrog-0.5.0-x86_64.AppImage
```

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.5.0-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.5.0-x86_64.tar.gz
cd StudyFrog-0.5.0
./bin/studyfrog
```

### Upgrading from 0.4.3
Open your vault as before. On the first start the files of `~/.config/studyfrog` (settings, provider keys, profiles) are
**copied** into `~/.local/share/studyfrog/config`; the old files stay where they are, a file that already exists in the
new place is never overwritten, and `secrets.toml` ends up readable by you only. From then on the new place is used.
The assistant's call log moves from `~/.local/state/studyfrog/agent.log` to `~/.local/share/studyfrog/logs/agent.log`.
