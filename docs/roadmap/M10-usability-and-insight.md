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
    - TASK-0154 Roadmap tab
    - TASK-0155 Board, Calendar and Reports tabs scoped to the stack
  - **STORY-0065** Import and export in JSON and other formats
    - TASK-0156 Specify the interchange formats
    - TASK-0157 Export a library or stack
    - TASK-0158 Import with preview and duplicate handling
    - TASK-0159 Import and export screens
  - **STORY-0066** Built-in wiki, FAQ and tutorials
    - TASK-0160 Plan the help content
    - TASK-0161 Help viewer with navigation and search
    - TASK-0162 User-added and user-edited help pages
    - TASK-0163 Assistant proposals for user help pages
    - TASK-0164 Write the vision, getting started and FAQ pages
    - TASK-0165 Guided tutorials
  - **STORY-0067** Importers for Anki and Obsidian
    - TASK-0166 Anki importer
    - TASK-0167 Obsidian vault importer
    - TASK-0168 Document how to write a format adapter
  - **STORY-0068** Diary for self-reported progress
    - TASK-0169 Specify the diary
    - TASK-0170 Diary entries in the data model, Markdown and index
    - TASK-0171 Diary screen
    - TASK-0172 Prompt for an entry after a rehearsal
    - TASK-0173 Self-assessment against measured results
  - **STORY-0069** Stack detail view like Jira's issue view
    - TASK-0174 Specify the detail view
    - TASK-0175 Detail view layout with the details panel
    - TASK-0176 Open an item in a side panel
    - TASK-0177 Activity log of reviews, edits and status changes
  - **STORY-0070** Rehearsal modes for countdown and count up
    - TASK-0178 Specify the rehearsal modes
    - TASK-0179 Countdown and count up in the run state machine
    - TASK-0180 Mode and timer in run setup and on the rehearsal screen
    - TASK-0181 Mode and time in the review and the reports

**Order:** statuses (0061) and the tab bar (0060) first, because the stack pages, library and calendar use them; notifications
(0055), the vault name (0056), the rehearsal modes (0070), import and export (0065) with its importers (0067), help (0066) and the diary (0068) are
independent; familiarity (0058) before its reports and scheduling use (0059); the library list and query (0057) before the stack
sub-views (0064) and the side panel of the detail view (0069); the stack overview (0063) before its sub-views.

**Accepted by the owner on 2026-11-02:** the vault name in the bar (reversing STORY-0041), the status names, the settings
tabs, the calendar behaviour (next reviews from the schedule, movable by drag and drop, a moved item stays out of the queue
until its new day, a warning past an exam or due date), familiarity as a float from 0.0 to 1.0, the library as a copy of
Jira's list view with an SQL-like search, a stack overview (Jira project view) apart from the stack detail view (Jira issue
view), Anki and Obsidian as the first importers with review history imported and the schedule adapted (as many formats as
possible later), export of review history by checkbox, help pages that are English and read-only plus user pages linked
to the assistant's proposals, and a diary with every rating.

**Also accepted on 2026-11-02:** the export asks for its mode in a modal (default without scheduling; advanced, for now
only to include scheduling), diary entries may carry pictures, sub-stacks are both a Summary section and a tab, names with
commas are allowed (BUG-0001), and familiarity means how well the user knows an item: it grows a little per view, grows
much with easier ratings, falls with harder ones, grows with shorter view time, falls slowly without contact while the item
is not Complete and its exam date has not passed, and is a sub-criterion of scheduling that affects selection, order, the
planned day and the scheduler's interval, and a per-item report. Rehearsal gets two more modes, countdown and count up
(STORY-0070).

**Decisions still open:** how strong familiarity's effect on the interval is by default (set by simulation), what happens
to the open item when a countdown ends (TASK for the modes' specification), the overview and detail layouts and whether
Goals and Forms tabs are wanted, and the quoting format for names with commas (a proposal in BUG-0001).

**Exit criteria:**
- Notifications can be marked read, deleted and filtered, and survive a restart
- The bar names the open vault
- The library is a sortable list with filters and a query syntax; 10k items scroll and filter within the frame budget
- A stack opens as an overview with tabs, and its own details open in a detail view or side panel
- Every item has a status, and finished items are not planned on the calendar
- After a rehearsal the next reviews show on the calendar per stack and day, and an entry can be moved to another day
- Familiarity values (0.0 to 1.0) are shown per item in reports and on Home with reasons, steer the queue, the calendar planner and the scheduler's intervals, and their error on the simulated history is recorded
- Settings are in tabs and every setting appears once
- A library or stack exports to JSON, with scheduling if the advanced mode asks for it, and imports again without loss; Anki (with its history) and Obsidian import
- The app has a help area with the vision, a FAQ and tutorials, a first-run tutorial and the user's own pages, which the assistant can propose edits to
- BUG-0001 is fixed
- Runs can be set up as a countdown or a count up, and the review shows the mode and the time used
- The diary records self-reported progress and a report sets it against measured results
