# Priority list

The order in which Claude works on the planned items of phase M10 (0.3.0), kept by hand next to the generated
[BOARD.md](BOARD.md). The reasons and the dependencies are in
[docs/roadmap/M10-usability-and-insight.md](docs/roadmap/M10-usability-and-insight.md). Tick an item when it is done and
verified (`scripts/check.sh` passes), and report it to the owner. The owner may reorder the list at any time.

Last updated: 2026-11-03 (0.3.1 released). Phase M10 was released as 0.3.0 on 2026-11-03; the diary (item 20) moved to phase M11 (0.4.0) at the
owner's request and is listed under Next.

## Wave 1: start now
- [x] 1. [BUG-0001](docs/issues/bugs/BUG-0001-teacher-and-subject-names-with-a-comma-cannot-be-used.md) Names with commas cannot be used
- [x] 2. [TASK-0143](docs/issues/tasks/TASK-0143-specify-the-statuses-their-names-and-their-transitions.md) Specify the statuses
- [x] 3. [TASK-0142](docs/issues/tasks/TASK-0142-settings-tab-bar-and-domain-grouping.md) Settings tab bar
- [x] 4. [TASK-0128](docs/issues/tasks/TASK-0128-vault-name-next-to-the-studyfrog-name-in-the-bar.md) Vault name in the bar
- [x] 5. [TASK-0135](docs/issues/tasks/TASK-0135-specify-the-familiarity-value.md) Specify the familiarity value

## Wave 2: foundations
- [x] 6. TASK-0144 and TASK-0145 Statuses in the data model and the UI
- [x] 7. TASK-0136 Record view and action time
- [x] 8. TASK-0137 Familiarity as a pure function, and TASK-0141 its factor on the FSRS interval (on from the start)
- [x] 9. TASK-0126 and TASK-0127 Notification records and the notification center
- [x] 10. TASK-0129 Specify the library redesign and the query syntax
- [x] 11. TASK-0179 and TASK-0180 Rehearsal modes: specification and engine

## Wave 3: the big screens
- [x] 12. TASK-0130 to TASK-0134 Library: query, list, search bar, saved filters, forms out of the page
- [x] 13. TASK-0146 to TASK-0148 Calendar: plan the next reviews, move them, drag and drop
- [x] 14. TASK-0138 to TASK-0140 and TASK-0184 Familiarity in reports, Home and queues, and the replay
- [x] 15. TASK-0149 to TASK-0151 Stack overview: specification, shell, Summary

## Wave 4: the rest
- [x] 16. TASK-0152 to TASK-0156 Stack sub-views
- [x] 17. TASK-0181 and TASK-0182 Rehearsal mode screens and reports
- [x] 18. TASK-0157 to TASK-0160 Import and export, then TASK-0167 to TASK-0169 Anki and Obsidian
- [x] 19. TASK-0161 to TASK-0166 Help: wiki, FAQ, tutorials
- [ ] 20. ~~TASK-0170 to TASK-0174 Diary~~ moved to 0.4.0 (see below)
- [x] 21. TASK-0185 to TASK-0187 Create dialog like Jira and defaults for new items

## Next: phase M11 (0.4.0)
Proposed by Claude on 2026-11-03 after the owner's test of 0.3.1; the owner may reorder. Reasons and dependencies are in
[M11-diary](docs/roadmap/M11-diary.md).

**Wave 1: small and painful now**
- [x] BUG-0009 PDF lists name the file (done)
- [x] 1. BUG-0007 Enter in a list does not continue it, BUG-0008 Bullets sit higher than their text
- [x] 2. TASK-0199 Open the new item from its "created" notification
- [x] 3. TASK-0200 Auto-save token by token (on a whitespace, at the latest after 5 seconds) and no Save button, then TASK-0201 "Saved" toast when leaving the editor
- [x] 4. TASK-0209 Tell the user which layouts and formats the importers read

**Wave 2: new capabilities**
- [ ] 5. TASK-0208 CSV import with column mapping (all problems at once)
- [ ] 6. TASK-0205 Write the standard prompts (they hand-hold an AI to files StudyFrog reads), TASK-0206 Prompts screen (copy and export)
- [ ] 7. TASK-0197 Tables, TASK-0215 Underline and colour, TASK-0216 Table of contents, TASK-0198 Text macros, TASK-0214 Code highlighting
- [ ] 8. TASK-0207 Run a prompt with the assistant
- [x] 8b. EPIC-027 Appearance: TASK-0221 (spec), TASK-0222 (dark, light or system), TASK-0223 to TASK-0227 are all built; the check of every screen in both themes (TASK-0226) is done except the dialogs and the PDF viewer

**Wave 3: larger, specify first**
- [ ] 9. TASK-0210 Decide the stack folder and reference rules (ADR; the owner: a folder can be marked as a stack, an item can be referenced by several stacks)
- [ ] 10. TASK-0211 to TASK-0213 and TASK-0217 (references, the stack mark and the per-stack schedule: TASK-0218, TASK-0219, TASK-0220), then TASK-0202 (views spec), TASK-0203 and TASK-0204
- [ ] 11. TASK-0195 Calm the remaining screens (STORY-0073)
- [ ] 12. TASK-0170 to TASK-0174 Diary
- [x] TASK-0196 Navigation history, BUG-0005 Exit button cut off, BUG-0006 Roadmap too small (done on main after 0.3.1)

## Feedback after 0.3.0 (released as 0.3.1 on 2026-11-03, EPIC-025)
- [x] BUG-0002 Padding, BUG-0003 Create dialog header, BUG-0004 Home figures
- [x] TASK-0188 Ask before discarding a filled Create dialog
- [x] TASK-0189 Red Exit button
- [x] TASK-0191 Suggest Anki and Obsidian files to import
- [x] TASK-0190 Own title bar (the owner tried it on 2026-11-03)
- [x] TASK-0192 Calmer UI, first part: library, stack page, Home, rehearsal setup, settings and help (`docs/specs/ui-calm.md`); accepted by the owner, the rest is TASK-0195
- [x] TASK-0193 Space between content and scroll bars, TASK-0194 Rehearsal progress bar against the open items
