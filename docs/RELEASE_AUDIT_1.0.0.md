# Regression audit and release check: StudyFrog 1.0.0

- **Date:** 2026-11-22
- **Audited commit:** `3c57dfbf8f7dc59d5ffd8556123784b5e094c31f` (branch `claude/friendly-knuth-bk4npp`, clean tree, before the release edits).
- **Release commit:** the `chore(release): 1.0.0` commit on this branch, which adds this file. Not tagged.
- **Scope:** RELEASING.md steps 1, 4, 5 and 6 (gate, changelog, version, package). Steps 2 and 3 were not run (see section 6).

## 1. Gate

| Run | Tree | Result |
|---|---|---|
| Baseline | `3c57dfb`, no release edits | `scripts/check.sh` exit 0 |
| Final | release tree (version 1.0.0, one test fix, section 1.1) | `scripts/check.sh` exit 0 |

Both runs used `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUST_MIN_STACK=16777216`. The gate steps are: `cargo fmt --check`, repo-lint fmt, clippy and tests, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, the 250-line limit, dependency direction, no inherent `impl` blocks, issue front matter, and the board check.

- **Tests:** 2232 passed, 0 failed, 0 ignored in both runs (554 result sections). Totals are not broken down per crate; the run groups tests by test binary.
- **Flakes:** none seen. `sidecar_process` passed in both runs and did not need a rerun.

### 1.1 Failure found by the final gate

The first final-gate run failed in `crates/studyfrog-ui/tests/chrome_views.rs`, test `the_help_button_opens_help_and_the_profile_menu_closes_the_vault`. The test asserted the label `StudyFrog 0.`, which is the old version. This was a real defect caused by the version bump, not a flake. The fix builds the expected label from `CARGO_PKG_VERSION`, as the app's menu does (`crates/studyfrog-ui/src/view/menus.rs`). The test then passed (10 of 10), and the full gate was rerun and passed. The fix touches only a test file; no application code changed after packaging.

## 2. Open issues (status `in_progress` or `blocked`)

No item is `in_progress`. Three are `blocked`, all in EPIC-067:

| Item | Title | Status | Blocks release? |
|---|---|---|---|
| EPIC-067 | Performance benchmarking and continuous improvement | blocked | Yes, per RELEASING.md step 3 (reference benchmark runs on the owner's machine). Not a code defect. |
| STORY-0201 | Reference runs on the owner's machine, with the machine's identity recorded | blocked | Same |
| STORY-0203 | Provisional budgets replaced by measured baselines on the owner's machine | blocked | Same |

## 3. Other open work against the v1.0.0 roadmap

These are not `in_progress` or `blocked`, but they decide whether the roadmap's milestones 14.1 to 14.4 are met (`docs/v1.0.0-roadmap.md`, acceptance: deliver EPIC-056 to EPIC-063).

- **Roadmap stories STORY-0165 to STORY-0187 (23):** 7 done (0165, 0167, 0173, 0176, 0182, 0183, 0185); 16 in review (0166, 0168 to 0172, 0174, 0175, 0177 to 0181, 0184, 0186, 0187).
- **Epics:** EPIC-066 done. EPIC-056, 057, 059, 060, 061, 062, 063 and 064 in review. EPIC-058 (FrogScript wiki) and EPIC-065 in backlog; EPIC-058's stories 0171 and 0172 are in review.
- **Judgement:** against the roadmap these are not finished. The review items are built and tested headless; they are not yet claimed as done. The owner must either accept them as 1.0.0 scope or hold the tag. This is a scope decision, not a test failure.
- **Window checks still open** (from STATUS.md and the story notes): STORY-0150, 0151 and 0152 (remaining criteria in ScriptStudio and the console); STORY-0177, 0192 and 0194 (focus, calendar movement, reminder peek); STORY-0186 and 0187 (mock exam view, with a real model); STORY-0198 (custom run and report).
- **Model grading** has been tested with a fake model only. No real language model was available here.

## 4. Board observation (not changed)

`BOARD.md` line 7 shows `Review (1)` and `Blocked (0)`, but the issue files hold 43 items in `review` and 3 in `blocked`. `scripts/board.sh --check` passes, so the header is apparently not checked against the files. Not changed in this release; `scripts/board.sh` should be looked at.

## 5. Packaging

`scripts/package-appimage.sh` (release build, then AppDir, tarball, appimagetool).

| Artifact | Path | Size (bytes) | SHA-256 | Status |
|---|---|---|---|---|
| Release binary, not stripped | `target/release/studyfrog` | 297,107,088 | n/a | built |
| Stripped binary in AppDir | `target/appimage/StudyFrog.AppDir/usr/bin/studyfrog` | 51,909,720 | n/a | built |
| Tarball | `target/appimage/StudyFrog-1.0.0-x86_64.tar.gz` | 22,151,533 | `114bb3efb65ac23c02c7552d45e9411de0be393a9f79b21f503dd24952489281` | built; `sha256sum -c` OK |
| AppImage | `target/appimage/StudyFrog-1.0.0-x86_64.AppImage` | n/a | n/a | **not produced** |
| appimagetool 1.9.0 (downloaded by the script) | `target/tools/appimagetool-1.9.0.AppImage` | 12,632,352 | `46fdd785094c7f6e545b61afcfb0f3d98d8eab243f644b4b17698c01d06083d1` | downloaded; the script pins a version but not a checksum |

- **Why there is no AppImage:** appimagetool printed `desktop-file-validate command is missing, please install it` and exited 1 (the script runs with `set -e`). `desktop-file-utils` is not installed on this machine. I did not install it: the repo permits only the appimagetool download.
- **Headless smoke test:** the packaged binary ran under `xvfb-run` with `--screenshot ... --screen home --size 1280x800`. It exited 0, wrote a 70,377-byte PNG, and the image shows the welcome screen. This is not the desktop start check of RELEASING.md step 6.
- **Tarball contents:** `bin/studyfrog`, the desktop entry, the icon, README, licences, CHANGELOG and `RELEASE_NOTES.md`.
- **Disk:** only 2.6 GB were free at the start. I deleted `target/debug` (55 GB of stale, regenerable build output, gitignored). The gate rebuilt what it needed.
- **Nothing was committed from `target/`, and `dist/` was not created.**

## 6. Not run

- **RELEASING step 2** (`cargo test --release -p studyfrog-candle`): needs an installed language model. None is present (no GGUF file, no MiniLM model directory found).
- **RELEASING step 3** (benchmarks): owner's machine only, per the task.
- **RELEASING step 6, desktop start** of the AppImage with a vault open: needs a real desktop session and the AppImage, which was not produced.

## 7. Decisions and actions for the owner before the tag

1. **Scope:** accept the 16 review stories and the review epics as part of 1.0.0, or hold the tag until they are done (section 3).
2. **AppImage:** install `desktop-file-utils` on the packaging machine, then rerun `scripts/package-appimage.sh`. Record the AppImage SHA-256 beside it.
3. **Desktop check:** start the AppImage on a real desktop and open a vault (RELEASING step 6).
4. **Owner's-machine checks:** the reference benchmark runs and the baselines (EPIC-067), and the heavy candle tests with the model installed (RELEASING step 2).
5. **Window checks:** the list in section 3.
6. **Tag and publish:** tag name `v1.0.0` (annotated, as RELEASING.md step 7 says), whether to publish the GitHub release, and whether to upload the AppImage, tarball and checksums. None of this was done here.
7. **Merge:** merging `claude/friendly-knuth-bk4npp` into the default branch is also the owner's decision.
8. **Board header:** section 4.

Priority list: the v1.0.0 release item (`priority-list.md`, milestone 14.5) is not ticked, because the owner has not verified it.
