# M11: Diary, a calmer UI, writing and prompts (0.4.0)

**Goal:** The user records how their learning feels and sees it next to what StudyFrog measured; writing, saving and organizing are smoother; prompts and clearer imports help bring material in and out.

**Entry criteria:** 0.3.0 released (M10 exit met, without the diary). The owner moved the diary out of 0.3.0 on 2026-11-03.

## Epics, stories, tasks

- **EPIC-024** Diary and a calmer UI for 0.4.0
  - **STORY-0068** Diary for self-reported progress (from EPIC-023; its design notes and the owner's answers of 2026-11-02 stand)
    - TASK-0170 Specify the diary
    - TASK-0171 Diary entries in the data model, Markdown and index
    - TASK-0172 Diary screen
    - TASK-0173 Prompt for an entry after a rehearsal
    - TASK-0174 Self-assessment against measured results
  - **STORY-0073** A calmer look on every screen
    - TASK-0195 Calm the remaining screens
    - TASK-0196 Navigation history (done), BUG-0005 Exit button cut off (done), BUG-0006 Roadmap too small (done)
- **EPIC-026** Writing, saving, organizing and prompts for 0.4.0 (added after the owner's test of 0.3.1 on 2026-11-03)
  - **STORY-0074** Markdown editor, formatting, macros and lists
    - TASK-0197 Tables and more Markdown in the editor and the preview
    - TASK-0198 Text macros that expand into Markdown
    - TASK-0214 Code blocks with language highlighting
    - TASK-0215 Underline and text colour
    - TASK-0216 Automatic table of contents
    - BUG-0007 Enter in a list does not continue the list
    - BUG-0008 Bullet points sit higher than their text
  - **STORY-0075** Create and edit without losing work
    - TASK-0199 Open the new item from the notification that says it was created
    - TASK-0200 Auto-save in the editor and no Save button
    - TASK-0201 Item saved toast when leaving the editor
  - **STORY-0079** Stack folders and items referenced by several stacks (the owner's decisions of 2026-11-03)
    - TASK-0210 Decide the stack folder and reference rules (ADR and spec)
    - TASK-0211 The referenced_by field in the file format, the index and the vault checks
    - TASK-0212 Services follow the membership
    - TASK-0213 References in the interface
    - TASK-0217 Mark a folder as a stack or as a normal folder
    - TASK-0218 Schedules, due dates and reviews per stack
    - TASK-0219 Familiarity and difficulty per stack, averaged on the item
    - TASK-0220 Export formats TOML and RON next to JSON
  - **STORY-0076** Organize the vault as folders and files
    - TASK-0202 Specify the folders and items views
    - TASK-0203 Folder operations in the service (create, rename, move, delete)
    - TASK-0204 Folders view and Items view in the application
  - **STORY-0077** Prompts for AI agents
    - TASK-0205 Write the standard prompts
    - TASK-0206 Prompts screen with copy and export
    - TASK-0207 Run a prompt with the connected assistant and an attached file
  - **STORY-0078** Imports that explain themselves
    - TASK-0208 CSV import with column mapping
    - TASK-0209 Tell the user which layouts and formats the importers read

## Priority order
Proposed by Claude on 2026-11-03; the owner may reorder.

1. **Small and painful now:** BUG-0007, BUG-0008, TASK-0199, TASK-0200 and TASK-0201 (writing and saving), TASK-0209 (say what imports expect).
2. **Next:** TASK-0208 (CSV mapping, with all problems at once), TASK-0205 and TASK-0206 (prompts that hand-hold an AI, and copying them), TASK-0197, TASK-0215 and TASK-0216 (tables, underline and colour, table of contents), TASK-0198 (macros), TASK-0214 (code highlighting), TASK-0207 (prompts through the assistant).
3. **Larger, decide first:** TASK-0210 (the ADR on stack folders and references) before anything else in this wave; then TASK-0211 to TASK-0213, TASK-0217, TASK-0218, TASK-0219 and TASK-0220 (the model change; vault files need no migration, the database does), TASK-0202, TASK-0203 and TASK-0204 (folders and items views), TASK-0195 (the remaining screens). The reference part is modest, but the owner's per-stack schedule (TASK-0218 and TASK-0219) changes how reviews and schedules are stored and queued: the owner should decide after the ADR whether that part stays in 0.4.0 or moves to 0.5.0, with the references and the stack mark shipping first.
4. **The diary:** TASK-0170 first (the specification), then TASK-0171, TASK-0172, TASK-0173 and TASK-0174 in order.

Dependencies: TASK-0205 and TASK-0208 use the layouts of TASK-0209; TASK-0201 needs TASK-0200; TASK-0198 needs the syntax of TASK-0197, TASK-0214, TASK-0215 and TASK-0216; TASK-0211 to TASK-0213, TASK-0217, TASK-0218 and TASK-0219 need TASK-0210 and follow each other in order (TASK-0218 and TASK-0219 need TASK-0212); TASK-0202 needs TASK-0210; TASK-0203 and TASK-0204 need TASK-0202, TASK-0211 and TASK-0217.

**Exit criteria:**
- Enter continues lists, tables and macros work, the editor saves by itself and says so, and a new item opens from its notification
- The vault has a Folders view and an Items view
- Prompts can be copied, exported and run through the assistant; imports say which layouts they read and CSV files can be mapped
- The diary records self-reported progress, with ratings, free text, pictures and links to stacks, items, runs and exams
- After a rehearsal the user is offered an entry
- A report sets the self-assessment against measured results
