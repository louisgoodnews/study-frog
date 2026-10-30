---
id: TASK-0067
type: task
title: Linux packaging (AppImage)
status: in_progress
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
- [ ] `scripts/package-appimage.sh` assembles an AppDir and packs an AppImage
- [ ] A CI workflow checks the repository and builds the AppImage as an artifact
- [ ] The AppImage was built and started

## Design Notes
The AppDir is assembled and verified here; packing needs `appimagetool`, which the script downloads, and downloading and running that tool was refused by the auto-mode permission check in this session, so the AppImage itself has not been built, and the workflow has been linted (`actionlint`) but not run.

## Dependencies
TASK-0066.

## Open Questions
None.
