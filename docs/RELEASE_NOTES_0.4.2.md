# StudyFrog 0.4.2

Rehearsal card text wrapping and vertical scrollability, automatic recovery of unquoted commas in LLM-generated CSV imports, outer and inner scrollable containers with dynamic review-area stretching on the rehearsal result view, and always-visible suggested ratings and reading metrics on every reviewed card. The full list is in `CHANGELOG.md` under `[0.4.2]`.

## What is new
- **Rehearsal Card Text Wrapping & Unquoted CSV Comma Recovery (`BUG-0012`).**
  - Prompts and answers in the rehearsal card view now enforce `TextWrapMode::Wrap` across markdown spans and sit inside a dedicated vertical `ScrollArea` so long cards wrap and scroll cleanly.
  - Both CSV importers (`studyfrog-ext-example` and `studyfrog-service::xfer_csv_map`) now detect when a comma-separated row in a headed file (such as 6-column `stack,kind,title,front,back,tags` output from Google Gemini or other LLMs) contains unquoted commas inside `front` or `back`, anchoring the leading metadata columns and trailing `tags` column and rejoining the middle segments so questions and answers are never silently truncated at the first comma.
- **Rehearsal Result View Scrollability, Stretched Review Area & Suggested Rating Visibility (`BUG-0013`).**
  - The entire rehearsal summary screen (`Run finished` / `Run stopped`) is now wrapped in an outer vertical `ScrollArea` so all sections remain reachable at any window size.
  - The `Review` section dynamically stretches its height with the viewport (replacing the previous fixed `140 px` cap) while scrolling its item cards inside an inner vertical `ScrollArea`.
  - Every card in the `Review` list now always displays `Suggested rating:` alongside the reading word count and time (`{words} words in {s} s`), even when the reading pace matches your chosen rating and no rating change is needed.

## Verification
- **Automated quality gates:** All 10 repository checks in `scripts/check.sh` (`cargo fmt --check`, `repo-lint`, `cargo clippy -D warnings`, workspace unit/integration/UI tests, doc-tests, 250-line file limit, dependency direction, zero inherent `impl` blocks, issue front-matter lint, and `BOARD.md` sync) pass on the release commit.

## Install
StudyFrog is for Linux on x86_64. It needs OpenGL, and X11 or Wayland, from your system; nothing else. Download the
files of your choice from the assets of this release.

### AppImage (one file, nothing to unpack)
```sh
sha256sum -c StudyFrog-0.4.2-x86_64.AppImage.sha256   # prints "OK" when the download is intact
chmod +x StudyFrog-0.4.2-x86_64.AppImage
./StudyFrog-0.4.2-x86_64.AppImage
```

### Tarball (`.tar.gz`)
```sh
sha256sum -c StudyFrog-0.4.2-x86_64.tar.gz.sha256     # prints "OK" when the download is intact
tar -xzf StudyFrog-0.4.2-x86_64.tar.gz
cd StudyFrog-0.4.2
./bin/studyfrog
```

### Upgrading from 0.4.1
Nothing to migrate: open your vault as before. If you previously imported a CSV with unquoted commas whose cards were truncated, re-importing the CSV file will import the complete card text.
