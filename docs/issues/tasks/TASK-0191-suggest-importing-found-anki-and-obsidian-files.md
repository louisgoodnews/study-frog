---
id: TASK-0191
type: task
title: "Suggest importing found Anki and Obsidian files"
status: done
epic: EPIC-025
parents: [STORY-0072]
estimate: 5
labels: [m10, ui, service]
acceptance_criteria:
  - "The import dialog looks in the usual places for Anki profiles and Obsidian vaults and lists the Obsidian vaults and Anki packages it found (with the number of Markdown files or the size) and names Anki profiles with the hint to export a package"
  - "Choosing a vault or package fills in the preview as if it had been picked; nothing is read beyond a name and a count until the user chooses"
  - "Nothing is found or the places cannot be read: the dialog says so and shows the normal picker"
---

## Context
The owner tested 0.3.0 for a short while on 2026-11-03 and sent five screenshots with a list of observations: missing padding, a crowded Create dialog header, no confirmation when cancelling a filled dialog, no import suggestions, hard-to-read window buttons, no in-app exit button, a cramped streak and an overall dense UI.

## Description
Look for an Anki and Obsidian installation on the machine and suggest importing from them.

## Acceptance Criteria
- [x] The import dialog looks in the usual places for Anki profiles and Obsidian vaults and lists the Obsidian vaults and Anki packages it found (with the number of Markdown files or the size) and names Anki profiles with the hint to export a package
- [x] Choosing a vault or package fills in the preview as if it had been picked; nothing is read beyond a name and a count until the user chooses
- [x] Nothing is found or the places cannot be read: the dialog says so and shows the normal picker

## Design Notes
Built as `studyfrog_service::xfer_discover` and `transfer_found`: Obsidian vaults are read from Obsidian's own `obsidian.json` (config, Flatpak and snap paths) with a count of Markdown files; `.apkg` files are listed from Downloads, Documents and Desktop; Anki profiles are only named, because Anki's live collection cannot be read (locked while Anki runs, and the importer reads the legacy package format), so the entry tells the user to export a package. Acceptance changed accordingly. Planned places were: Anki `~/.local/share/Anki2/<profile>/collection.anki2` and the Flatpak path; Obsidian's list of vaults in `~/.config/obsidian/obsidian.json` and the Flatpak and snap paths. The search is read-only and runs when the dialog opens.

## Dependencies
None.

## Open Questions
None.
