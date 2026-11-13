---
id: TASK-0125
type: task
title: "Publish a tar.gz and its checksum with every release"
status: done
epic: EPIC-022
parents: [STORY-0054]
estimate: 1
labels: [m9, pdf]
acceptance_criteria:
  - "The packaging script writes StudyFrog-VERSION-x86_64.tar.gz with the binary, desktop entry, icon, licences, README, changelog and release notes, and a .sha256 file beside it"
  - "The release workflow uploads the tar.gz and its checksum next to the AppImage"
  - "The 0.2.0 release carries both files"
---

## Context
The owner asked on 2026-10-31 for a tar.gz and a checksum file in the release as well.

## Description
The archive is made from the same stripped binary as the AppImage.

## Acceptance Criteria
- [x] The packaging script writes StudyFrog-VERSION-x86_64.tar.gz with the binary, desktop entry, icon, licences, README, changelog and release notes, and a .sha256 file beside it
- [x] The release workflow uploads the tar.gz and its checksum next to the AppImage
- [x] The 0.2.0 release carries both files

## Design Notes
The 0.2.0 files were attached by hand to the release the workflow had already created, using the binary from its AppImage.

## Dependencies
TASK-0124.

## Open Questions
None.
