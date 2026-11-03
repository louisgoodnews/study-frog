# M10: Usability and insight (0.3.0)

**Goal:** StudyFrog is easier to find your way around, helps the user learn it, and tells the user what to study next.

**Entry criteria:** 0.2.0 released (M9 exit met). Requested by the owner on 2026-11-02; the Jira list view, project view and
issue view are the visual reference for the library, the stack overview and the item view.

## Epics, stories, tasks

- **EPIC-023** Usability and insight for 0.3.0
  - **STORY-0055** Notification management
    - TASK-0126 Notification records with ids, read state and persistence
    - TASK-0127 Notification center panel
  - **STORY-0056** Current vault in the top bar
    - TASK-0128 Vault name next to the StudyFrog name in the bar
  - **STORY-0057** Library redesign
    - TASK-0129 Specify the library redesign and the query syntax
    - TASK-0130 Search query parser and evaluator
    - TASK-0131 Item list with columns and sorting
    - TASK-0132 Search bar with Standard and Query modes
    - TASK-0133 Saved filters and bulk actions
    - TASK-0134 Move card creation and import out of the page
  - **STORY-0058** Familiarity value
    - TASK-0135 Specify the familiarity value
    - TASK-0136 Record how long items are viewed and acted on
    - TASK-0137 Familiarity as a pure function with benchmarks
  - **STORY-0059** Familiarity in reports, on Home and in scheduling
    - TASK-0138 Per-item familiarity in the reports
    - TASK-0139 Needs-attention card and least-familiar run order
    - TASK-0140 Familiarity in queue selection and the calendar planner
    - TASK-0141 Familiarity scales the scheduler's interval
    - TASK-0183 Research a model that combines familiarity with FSRS
    - TASK-0184 Replay familiarity against the user's review history for a verdict
  - **STORY-0060** Settings in tabs
    - TASK-0142 Settings tab bar and domain grouping
  - **STORY-0061** Statuses for stacks, notes, cards and questions
    - TASK-0143 Specify the statuses, their names and their transitions
    - TASK-0144 Status field in the data model and Markdown schema
    - TASK-0145 Change and show statuses in the UI
  - **STORY-0062** Calendar plans the next reviews and lets the user move them
    - TASK-0146 Plan the next reviews as calendar entries
    - TASK-0147 Move an entry to another day
    - TASK-0148 Calendar entries with drag and drop
  - **STORY-0063** Stack page with tabs and summary
    - TASK-0149 Specify the stack view redesign
    - TASK-0150 Stack page shell with header and tabs
    - TASK-0151 Summary tab
  - **STORY-0064** Stack sub-views for list, backlog, roadmap, board, calendar and reports
    - TASK-0152 List and Backlog tabs
    - TASK-0153 Sub-stacks tab
    - TASK-0154 Goals tab
    - TASK-0155 Roadmap tab
    - TASK-0156 Board, Calendar and Reports tabs scoped to the stack
  - **STORY-0065** Import and export in JSON and other formats
    - TASK-0157 Specify the interchange formats
    - TASK-0158 Export a library or stack
    - TASK-0159 Import with preview and duplicate handling
    - TASK-0160 Import and export screens
  - **STORY-0066** Built-in wiki, FAQ and tutorials
    - TASK-0161 Plan the help content
    - TASK-0162 Help viewer with navigation and search
    - TASK-0163 User-added and user-edited help pages
    - TASK-0164 Assistant proposals for user help pages
    - TASK-0165 Write the vision, getting started and FAQ pages
    - TASK-0166 Guided tutorials
  - **STORY-0067** Importers for Anki and Obsidian
    - TASK-0167 Anki importer
    - TASK-0168 Obsidian vault importer
    - TASK-0169 Document how to write a format adapter
  - **STORY-0068** Diary for self-reported progress (moved to EPIC-024, phase M11, on 2026-11-03)
    - TASK-0170 Specify the diary
    - TASK-0171 Diary entries in the data model, Markdown and index
    - TASK-0172 Diary screen
    - TASK-0173 Prompt for an entry after a rehearsal
    - TASK-0174 Self-assessment against measured results
  - **STORY-0070** Rehearsal modes for countdown and count up
    - TASK-0179 Specify the rehearsal modes
    - TASK-0180 Countdown and count up in the run state machine
    - TASK-0181 Mode and timer in run setup and on the rehearsal screen
    - TASK-0182 Mode and time in the review and the reports
  - **STORY-0071** Create dialog like Jira
    - TASK-0185 New items default to the highest difficulty and medium priority
    - TASK-0186 Create dialog laid out like Jira's
    - TASK-0187 Markdown editors in the create dialog that grow by dragging

**Withdrawn:** STORY-0069 (stack detail view) and TASK-0175 to TASK-0178 on 2026-11-02, because the existing item page is the
issue-like detail view for a note, a card, a question and a stack and the owner accepted it as it is. The numbers are not reused.

## Priority order

Proposed by Claude on 2026-11-02 from the dependencies between stories and from what hurts the user today; the owner may
reorder. The board's Ready column holds wave 1. WIP limit 3 applies to work in progress, so start the first three.

**Wave 1: start now (status `ready`).**
1. BUG-0001 Names with commas cannot be used (the owner cannot assign a professor to a subject today).
2. TASK-0143 Specify the statuses (decided; the stack pages, library and calendar all need statuses).
3. TASK-0142 Settings tab bar (the same tab widget serves the stack page).
4. TASK-0128 Vault name in the bar (small and independent).
5. TASK-0135 Specify the familiarity value (the formula is needed by reports, scheduling and the FSRS factor).

**Wave 2: foundations.** TASK-0144 and TASK-0145 statuses in data and UI; TASK-0136 view-time records; TASK-0137 familiarity
function and TASK-0141 its factor on the FSRS interval (on from the start); TASK-0126 and TASK-0127 notifications;
TASK-0129 library and query specification; TASK-0179 rehearsal modes specification and TASK-0180 its engine.

**Wave 3: the big screens.** TASK-0130 to TASK-0134 library (query, list, bar, filters, forms); TASK-0146 to TASK-0148 calendar
planner and drag and drop; TASK-0138 to TASK-0140 and TASK-0184 familiarity in reports, Home and queues, and the replay;
TASK-0149 to TASK-0151 stack overview (specification, shell, Summary).

**Wave 4: the rest.** TASK-0152 to TASK-0156 stack sub-views; TASK-0181 and TASK-0182 rehearsal mode screens and reports;
TASK-0157 to TASK-0160 import and export, then TASK-0167 to TASK-0169 Anki and Obsidian; TASK-0161 to TASK-0166 help;
TASK-0170 to TASK-0174 diary.

Dependencies behind the order: statuses and the tab widget before the stack pages and the calendar; the familiarity formula
before its reports and the FSRS factor; the library list and query before the stack sub-views; the stack overview before its
sub-views; the import and export engine before the importers.

**Accepted by the owner on 2026-11-02:** the vault name in the bar (reversing STORY-0041), the status names, the settings
tabs, the calendar behaviour (next reviews from the schedule, movable by drag and drop, a moved item stays out of the queue
until its new day, a warning past an exam or due date), familiarity as a float from 0.0 to 1.0, the library as a copy of
Jira's list view with an SQL-like search, a stack overview (Jira project view) apart from the existing stack editing page
(the issue-like detail view, unchanged), Goals as a bullet list the user edits and no Forms tab, Anki and Obsidian as the
first importers with review history imported and the schedule adapted (as many formats as possible later), the export
modal (default without scheduling; advanced, for now only to include scheduling), help pages that are English and read-only
plus user pages linked to the assistant's proposals, a diary with every rating and pictures, sub-stacks as a Summary section
and a tab, names with commas (BUG-0001), the rehearsal modes (the open item at the end of a countdown counts as not
answered), and familiarity: it grows a little per view, much with easier ratings, falls with harder ones, grows with shorter
view time, falls slowly without contact while the item is not Complete and its exam date has not passed, and is a
sub-criterion of scheduling that affects selection, order, the planned day and the scheduler's interval (more familiar items are scheduled more generously, through a relative factor on the FSRS interval with beta 2, clamp 0.25 to 2 and no change before the third review, on from the start; see docs/research/familiarity-and-fsrs.md; the 30 days once mentioned were an example, not a cap), and a per-item report.

**Decisions still open:** the stack overview layout (TASK-0149), whether a countdown may be extended, and the quoting format
for names with commas (a proposal in BUG-0001).

**Exit criteria:**
- Notifications can be marked read, deleted and filtered, and survive a restart
- The bar names the open vault
- The library is a sortable list with filters and a query syntax; 10k items scroll and filter within the frame budget
- A stack opens as an overview with tabs, apart from the existing detail page
- Every item has a status, and finished items are not planned on the calendar
- After a rehearsal the next reviews show on the calendar per stack and day, and an entry can be moved to another day
- Familiarity values (0.0 to 1.0) are shown per item in reports and on Home with reasons, steer the queue, the calendar planner and the scheduler's intervals, and their error on the simulated history is recorded
- Settings are in tabs and every setting appears once
- A library or stack exports to JSON, with scheduling if the advanced mode asks for it, and imports again without loss; Anki (with its history) and Obsidian import
- The app has a help area with the vision, a FAQ and tutorials, a first-run tutorial and the user's own pages, which the assistant can propose edits to
- Runs can be set up as a countdown or a count up, and the review shows the mode and the time used
- BUG-0001 is fixed
- ~~The diary records self-reported progress and a report sets it against measured results~~: moved to M11 (0.4.0) on
  2026-11-03, when the owner chose to release 0.3.0 without it.

**Status:** done; released as 0.3.0 on 2026-11-03 (notes in `docs/RELEASE_NOTES_0.3.0.md`).

**0.3.1 (EPIC-025):** fixes and additions from the owner's first use of 0.3.0 on 2026-11-03: BUG-0002 to BUG-0004, STORY-0072 with TASK-0188 to TASK-0194.
