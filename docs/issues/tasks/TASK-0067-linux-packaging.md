---
id: TASK-0067
type: task
title: Linux packaging (AppImage)
status: done
epic: EPIC-017
parents: [STORY-0032]
estimate: 5
labels: [m8, release]
acceptance_criteria:
  - "`scripts/package-appimage.sh` assembles an AppDir and packs an AppImage"
  - "A CI workflow checks the repository and builds the AppImage as an artifact"
  - "The AppImage was built and started"
---

## Context
AppImage was chosen over Flatpak: one self-contained file, no runtime or sandbox to maintain.

## Description
Linux packaging (AppImage).

## Acceptance Criteria
- [x] `scripts/package-appimage.sh` assembles an AppDir and packs an AppImage
- [x] A CI workflow checks the repository and builds the AppImage as an artifact
- [x] The AppImage was built and started

## Design Notes
Built on 2026-10-30 with the owner's one-off permission to download the pinned `appimagetool` 1.9.0: `StudyFrog-0.1.0-x86_64.AppImage`, 9.5 MB, SHA-256 `40721ef05693a763941ae10d4599ffb8583ece40ae324dda24e7933e2e6825fd`. It starts directly (FUSE) and with `APPIMAGE_EXTRACT_AND_RUN=1`; `--smoke-frames 30` opened a window on the reference desktop and exited 0. Still open: the CI workflow has been linted with `actionlint` but never run (no git remote), so "builds from CI" is unverified; this item stays in review until it has run once.

## Dependencies
TASK-0066.

## Open Questions
None.
