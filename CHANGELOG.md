# Changelog

All notable changes to StudyFrog are recorded here, newest first.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The project is pre-release, so
everything is under `Unreleased` until the first tagged version. Each entry names the work item
(`TASK-xxxx`) it belongs to where one exists.

## [Unreleased]

### Added
- **Saved classroom sessions: browser, resume, Markdown export and confirmed delete, with control-protocol inspection (`crates/studyfrog-ui`, TASK-0295, STORY-0211, EPIC-068)**: the Classroom screen lists saved sessions with title, topic, date and counts; Resume reloads one, Export writes `exports/classroom-<id>.md` in the vault, and Delete asks for confirmation. The control protocol reports the active and saved sessions under `classroom`. Tests: `studyfrog-ui/tests/classroom_saved_ui.rs`, `classroom_saved_view.rs`, `control_classroom.rs`.
- **Classroom assignments: panel, group thread, turn-in and graded view (`crates/studyfrog-ui`, `crates/studyfrog-core`, `crates/studyfrog-store`, `crates/studyfrog-service`, TASK-0294, STORY-0211, EPIC-068)**: the Classroom screen lists solo and group tasks with `assigned`, `submitted` and `graded` badges and rosters, beside the lecture chat (stacked below 720 px). The task drawer holds the group thread, where the user writes and the simulated teammates answer; the shared answer draft with Turn in, which calls no model; and Ask the teacher to grade. The grade view shows the score, the letter, each rubric point's verdict and sentence, and the feedback. `studyfrog-core::classroom_status` gives owners and work status. A grade now stores its rubric breakdown (`CriterionMark`, migration 34 `classroom_grade_criterion`). `studyfrog-service::classroom_group::group_message` stores the user's message before the teammates answer. Tests: `studyfrog-ui/tests/classroom_work_ui.rs` and `classroom_work_view.rs` (9 cases, fake local model), `studyfrog-service/tests/classroom_group_message.rs` (2 cases), `studyfrog-core/src/tests/classroom_status.rs` (3 cases), and the store round trip in `classroom_store.rs` with the criteria.)
- **Classroom screen: session form and lecture chat (`crates/studyfrog-ui`, TASK-0293, STORY-0211, EPIC-068)**: a Classroom screen (sidebar, under Study) starts a session from ticked stacks and PDFs, a chosen model connection and 0 to 3 simulated student personas. The lecture chat draws role badges for teacher, student and user with `palette_of` tokens, and controls continue the lecture, ask a student a question, set a task and send the user's message. Every model turn runs on the worker through the classroom service use cases. Tests: `studyfrog-ui/tests/classroom_ui.rs` (12 cases, fake local model, no network).
- **Turn-in, simulated peer answers and teacher grading with a keyword fallback (`crates/studyfrog-agent`, `crates/studyfrog-service`, TASK-0292, STORY-0210, EPIC-068)**: `classroom_submit::turn_in` locks a trimmed solo or group answer; `record_peer_answers` has the model write the answer of each simulated group beside the user's group, once, stored in one write or not at all. `classroom_grade::grade_submission` has the teacher grade the answer against the rubric (`lecture_grade`): points out of 100 from the rubric points met, a letter (A to F), per-criterion notes and summary feedback. The keyword check (`rubric_eval`) grades when the model call fails, its reply does not read, or the provider may not receive vault text. The points and summary are stored with the grade; the letter and the per-criterion notes are returned but not stored yet. `classroom_work::check_owner` is public. Tests: `studyfrog-agent/tests/lecture_grade.rs` (11 cases), `studyfrog-service/tests/classroom_{submit,submit_peers,grade,grade_refusals}.rs` (18 cases, shared helpers in `tests/classroom_work_common/`), fake models only.
- **Solo and group classroom tasks with rubric points and teammate contributions (`crates/studyfrog-agent`, `crates/studyfrog-core`, `crates/studyfrog-store`, `crates/studyfrog-service`, TASK-0291, STORY-0210, EPIC-068)**: the teacher sets a structured task (title, prompt, rubric points, solo or group) for one student or for groups of the user and the simulated students (`classroom_task::assign_task`); the model's JSON is parsed, and a fixed template about the current segment replaces it when it does not parse. `Assignment` gains `rubric`, stored in migration 33 (`classroom_rubric`). Before a group turns in, each simulated teammate adds one contribution to the group thread (`classroom_group::group_contributions`), stored as a chat turn; refused after turn-in, for solo tasks and for unknown groups. Group partitioning is pure (`lecture_groups`). Tests: `studyfrog-agent/tests/lecture_{task,groups,teammate}.rs` (12 cases), `studyfrog-service/tests/classroom_{task,group}.rs` (12 cases, fake model), `studyfrog-core/src/tests/classroom_assign.rs` (rubric case), `studyfrog-store/tests/classroom_store.rs` (migration 33).
- **Classroom turn orchestrator and service use cases (`crates/studyfrog-service`, TASK-0290, STORY-0209, EPIC-068)**: `classroom_turns` starts a session from a stack, notes or a lecture PDF (`start_session`), advances the lecture one segment (`advance_segment`), has a simulated student ask and the teacher answer (`simulated_question`) and handles a user message (`user_message`); each turn is written to the store before the next one is generated. `classroom_context` packs the prompt with the lecture budget, and `classroom_model` sends it through `agent::ask` to any provider, refusing a remote one without consent. `classroom_material` reads the sources; a PDF is listed under a fingerprint of its vault path. Tests: `studyfrog-service/tests/classroom_{start,lecture,provider}.rs` (12 cases, fake model).
- **Lecture prompt builders and context budgeter (`crates/studyfrog-agent`, TASK-0289, STORY-0209, EPIC-068)**: `lecture_prompt` builds the teacher's system and user messages (next segment, answer a question, check comprehension) and the rolling-summary prompt; `lecture_student` builds a simulated student's messages from a `Persona` (three presets: curious beginner, detail-oriented skeptic, practical applier) and lists the questions already asked; `lecture_budget` packs the summary (at most a quarter of the budget), recent turns (at most half, newest first, contiguous) and syllabus excerpts (the rest, last one cut short) into a character budget. Pure. Tests: `studyfrog-agent/tests/{lecture_prompt,lecture_student,lecture_budget}.rs` (24 cases, including empty lecture, lecture larger than budget and zero budget).
- **Classroom sessions record their vault sources (`crates/studyfrog-core`, `crates/studyfrog-store`, TASK-0288, STORY-0208, EPIC-068)**: a session lists the stacks, notes and PDFs its lecture is grounded in (`SourceRef`, `add_source` refuses a repeat), stored in the `classroom_source` table and reloaded in order.
- **Classroom session store: SQLite migration 32 and repository functions (`crates/studyfrog-store`, TASK-0288, STORY-0208, EPIC-068)**: normalized tables for sessions, participants, chat turns, solo and group assignments, group rosters, submissions and grades. `classroom_store` saves, loads, lists, updates and deletes whole sessions, each write in one transaction; deleting a session cascades to every child row. `export_session_markdown` renders the transcript, assignments, submissions and grades as Markdown (`classroom_markdown`, pure). Tests: `studyfrog-store/tests/classroom_store.rs` (7 cases, including reopen round-trip and cascading deletes) and `studyfrog-store/tests/classroom_export.rs` (1 case).
- **Classroom session domain types and pure transitions (`crates/studyfrog-core`, TASK-0287, STORY-0208, EPIC-068)**: `classroom`, `classroom_ops` and `classroom_work` define `ClassroomSession`, `Participant` (teacher, simulated student, user), `ChatTurn`, `Assignment` (solo or group), `GroupRoster`, `Submission` and `GradeReport` as plain data, with free functions that create a session, append turns, assign work, turn in an answer and record a grade. A refused operation returns a `ClassroomError` and leaves the session unchanged: grading before turn-in, a second turn-in (the answer is locked), a regrade and an out-of-range grade are rejected. Tests: `studyfrog-core/src/tests/classroom.rs` (12 cases).
- **Mock exam item format selection modal, 6-format generation, interactive answering, and deterministic objective grading (`crates/studyfrog-rehearsal`, `crates/studyfrog-agent`, `crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0207, EPIC-063)**: launching "Mock exam" or choosing a PDF under "Mock exam from a PDF" opens `Modal::MockExamFormat` to toggle any of six formats (`Open question`, `Fill in the gaps`, `Single select`, `Multiple select`, `Calculation`, `Case scenario`), configure counts or proportional weights and total questions with live validation and session persistence, generate format-aware questions via SLM or offline template synthesis (`curriculum_choices`, `curriculum_gaps`), answer with format-appropriate controls (text areas, inline Tab-navigable cloze gaps, single-choice radios, multi-choice checkboxes), and review results with per-format breakdowns, engine badges, and explicit multiple-select net partial-credit scoring (`MULTI_SELECT_RULE`). Tests: `studyfrog-rehearsal/tests/{exam_format,exam_objective_grade,curriculum_build,curriculum_material}.rs`, `studyfrog-agent/tests/exam_gen.rs`, `studyfrog-service/tests/{exam_gen,exam_save}.rs`, `studyfrog-ui/tests/{mock_exam_format_modal_ui,mock_exam_formats_answering_ui,mock_exam_graded_review_ui}.rs`.
- **Allocations per frame in the rehearsal card, reports and console are budgeted (`crates/studyfrog-ui`, EPIC-067, STORY-0205, TASK-0285)**: three dhat tests draw whole app frames headless on those screens and assert the count stays within a budget recorded when the task was done (console 855, rehearsal card 927, reports 1,275 allocations per frame; egui allocates inside each frame, so zero is not reachable). Tests: `studyfrog-ui/tests/alloc_console.rs`, `alloc_rehearsal_card.rs`, `alloc_reports.rs`; `dhat` is a dev-dependency of `studyfrog-ui` only.
- **Ollama setup assistant: hardware audit, local Ollama status, suggested models and a setup guide (`crates/studyfrog-service`, `crates/studyfrog-ui`, `crates/studyfrog-net`, STORY-0206, EPIC-055)**: Assistant > Connections > "Set up a local model" audits the processor, memory, graphics cards and the free disk of the models folder (unknown values stay unknown), asks the local Ollama service (127.0.0.1:11434 only) whether it is missing, not running, has no models or is ready, ranks a curated catalog (`data/ollama_catalog.toml`) by fit for this computer, and shows the setup steps with copyable commands. A chosen model is tested with one short prompt and saved as the local model only if it answers; nothing is downloaded, installed or started by the app. The same is in the control protocol as `audit`, `ollama status` and `ollama suggest`. Tests: `studyfrog-service/tests/ollama_audit.rs`, `ollama_fit.rs`, `ollama_stub.rs`, `studyfrog-ui/tests/ollama_panel_ui.rs`, `ollama_control.rs`.
- **Widget tree in the control protocol (`crates/studyfrog-ui`, STORY-0124, EPIC-038)**: `ui tree` answers the named widgets of the window's frame (role, label, value, enabled; at most 300), `ui find LABEL` the widgets labelled so, and `ui click LABEL` clicks one through AccessKit (a label with delete, remove or discard needs `--control-allow-destructive`). The window asks egui for its AccessKit tree only while a request waits; a headless instance refuses.
- **Achievements, graph, assistant, settings, match, folders and PDF open in the control protocol (`crates/studyfrog-ui`, STORY-0124, EPIC-064)**: `achievements [list]`, `achievements filter all|global|stack FOLDER`, `graph`, `assistant [state]`, `assistant send TEXT` (refused as not configured without a model), `settings [read]`, `settings slow-answer on|off`, `match [state]`, `match start`, `folders [list]`, `folders open PATH`, `pdf open PATH` (page count, or the reason the file did not open); each has a help row and a test.
- **`stack delete FOLDER` in the control protocol (destructive, `crates/studyfrog-ui`, STORY-0124, EPIC-064)**: runs the folders screen's delete confirmation; needs `--control-allow-destructive`; answers the deleted folder, or why the stack is still there.
- **`pdf find` answers the page count (`crates/studyfrog-ui`, STORY-0124, EPIC-064)**: the answer carries `page_count` beside the find results.
- **Cognitive profile lists each item's estimated difficulty (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0177, EPIC-060)**: the profile shows up to 20 items, hardest first, each with its title, folder, the mean empirical difficulty (1 to 10) over its timed answers and the answer count; the figure comes from the existing difficulty engine. Pure functions in `studyfrog-service::cognitive_items`. Tests: `studyfrog-service/tests/cognitive_items.rs`, `studyfrog-ui/tests/reports_cognitive_ui.rs` (`each_item_shows_its_title_and_estimated_difficulty`).
- **Cognitive profile compares the adaptive schedule with the fixed one (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0177, EPIC-060)**: for each item's second review, the interval and stability under the calibrated starting difficulty are set against those under the plain prior, replayed from the history; the first review gives the same stability under both. `schedule::timing_difficulty` is now shared by the live first-review path and the replay. Tests: `studyfrog-service/tests/schedule_compare.rs`, `studyfrog-ui/tests/reports_cognitive_ui.rs`.
- **Mock exam pass mark setting (`crates/studyfrog-rehearsal`, `crates/studyfrog-ui`, STORY-0187, EPIC-063)**: the graded exam's result has a "Pass mark" control (whole percent, 1 to 100, default 50) that changes the grade, the verdict and the per-topic colours at once, without regrading the answers. Tests: `studyfrog-ui/tests/mock_exam_ui.rs` (`the_pass_mark_is_a_setting_of_the_exam_that_moves_the_verdict_and_is_clamped`), `studyfrog-rehearsal/tests/grade.rs`.
- **Math drill hint and result: standard scores and confidence band (`crates/studyfrog-rehearsal`, `crates/studyfrog-ui`, EPIC-062)**: the drill has a Hint button that reveals the value of the next working row (up to three, the answer is not marked); the full solution also shows the person's z-score, T score, IQ-scale score and the 95% confidence band of the true score (`formula_report::score_report`, built on the existing confidence-interval and standard-score functions). Tests: `studyfrog-rehearsal/tests/formula_report.rs`, `studyfrog-ui/tests/math_drill.rs`.
- **Hover explanations for the navigation rail, link buttons, the Comments toggle and the transfer dialog's tabs (`crates/studyfrog-ui`, STORY-0189, STORY-0192, EPIC-057)**: each screen in the rail, each help page entry, the item-title and other link buttons, the PDF fit buttons, the rehearsal's Comments toggle (shortcut C) and the Export and Import tabs have a one-line explanation; `help tooltip Calendar` answers the rail's names. The guard test `tests/text_button_tooltips.rs` also scans selectable buttons and links. Test: `studyfrog-ui/tests/text_button_tooltips.rs`.
- **Control protocol drives the board, diary, reports and calendar views (`crates/studyfrog-ui`, STORY-0124, EPIC-038, EPIC-064)**: `board`, `board column NAME`, `board add TITLE [--column NAME]`; `diary [list [DATE [DATE]]]`, `diary open ULID`, `diary add DATE TITLE [--text T] [--minutes N] [--tags T] [--stacks S]`; `reports [TAB]` answers the figures of each of the nine tabs; `calendar view` and `calendar period` answer with the view, month and selected day, and `homework tab` with the tab in front. Help rows for all of them. Tests: `studyfrog-ui/tests/control_board.rs`, `control_diary.rs`, `control_reports.rs`, `control_calendar_view.rs`.
- **Cognitive profile charts: velocity curve, answer-time distribution and semantic accuracy trend (`crates/studyfrog-service`, `crates/studyfrog-ui`, BUG-0030, EPIC-060)**: Reports > Cognitive profile draws the learning velocity as a curve over cumulative answering minutes, the answer times as a 2-second bin distribution, and the running mean of the judged answers' semantic scores (or says why none is shown yet). Pure functions in `studyfrog-service::cognitive_curves`. Tests: `studyfrog-service/tests/cognitive_curves.rs`, `studyfrog-ui/tests/reports_cognitive_ui.rs`.
- **Homework planner shows the timetable and filters by subject (`crates/studyfrog-ui`, `crates/studyfrog-calendar`, BUG-0043, EPIC-056)**: the list has a Subject row ("All" and one chip per stack the homework belongs to) that the status tab keeps; the week's timetable sits beside the list (above it in a narrow window), and each item names the lessons on its due day. Control: `homework subject <stack folder>|all`. Tests: `studyfrog-calendar/tests/timetable.rs`, `homework_query.rs`, `studyfrog-ui/tests/homework_screen.rs`, `control_homework.rs`, `calendar_timetable.rs`.
- **`calendar session add` in the control protocol (`crates/studyfrog-ui`, STORY-0194, EPIC-064)**: `calendar session add TITLE --start "YYYY-MM-DD HH:MM" --minutes N` creates a calendar session through the planner's own new-session form and Save, so `reminders due` can report it; the answer is the created session. Test: `studyfrog-ui/tests/control_calendar_session.rs`.
- **Help links to the FrogScript reference (`crates/studyfrog-ui`, EPIC-058)**: the Help screen's page tree has a "FrogScript reference" button (with a vault open) that opens the reference in ScriptStudio, expanded, with the same panel as the Console's "Reference" button. Test: `studyfrog-ui/tests/help_wiki.rs`.

### Changed
- **The UI's `Msg` and `Effect` enums are split into per-area sub-enums (`crates/studyfrog-ui`, TASK-0296)**: `Msg` and `Effect` keep one wrapper variant per area (`Msg::Vault(MsgVault::..)`, `Effect::Rehearsal(EffectRehearsal::..)`), defined in `msg/` and `effect/`. Behaviour is unchanged; the worker log still names an effect by its variant, without the area wrapper.
- **Note editor: Split is a secondary "Preview below" toggle, under the text, not a tab beside it (`crates/studyfrog-ui`, EPIC-036, STORY-0114)**: the text keeps the reading measure in Write, Preview and Split (about 55 to 57 characters per line at 1280x800 and 960x640; beside each other the sides held 28 to 50). The view switch has no keyboard shortcut to keep. Tests: `studyfrog-ui/tests/editor_measure_ui.rs`, `editor_views.rs`.
- **Note text is set in the body font and a note opens in Write (`crates/studyfrog-ui`, EPIC-036, STORY-0114)**: the Markdown editor's text uses the body font instead of the monospace code font, so a line holds about 59 characters in Write at 1280x800 and 960x640 (about 49 before), inside the 55 to 75 band. A note opens in Write; Split is one click away and holds about 50 characters per line at 1280x800 and about 30 at 960x640, below the band, so EPIC-036 criterion 2 stays open. Test: `studyfrog-ui/tests/editor_measure_ui.rs`.
- **FSRS parameter optimization starts from the calibrated difficulty (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0177, EPIC-060)**: with the calibrated start on, the optimizer's replay of each item's first review uses the difficulty blended with the earlier first answers, as the live schedule does; with it off, the fit is unchanged. Tests: `studyfrog-service/tests/fsrs_optimize.rs` (`a_calibrated_first_state_differs_only_when_there_are_observations`).
- **One top-level Import button (`crates/studyfrog-ui`, STORY-0168, EPIC-057)**: the top bar has an "Import" button next to "+ Create" that opens the same transfer dialog on the Import tab, scoped to the selected stack when there is one; the Import entries in the items toolbar's More menu and the stack page's More actions are removed (Export stays there). The bar keeps every control at 960x640; the search field narrows to make room.
- **Previous rating on the rating button (`crates/studyfrog-ui`, STORY-0196, EPIC-065)**: the rating button of the item's previous rating has a "Last: <word>" line above it in the colour of its difficulty (red Difficult, orange Medium, green Easy); the line is in the body size, and no line shows for an item never rated.
- **Board headings count every work item of each status (`tools/repo-lint`, EPIC-058)**: the per-status counts in the board's headings include every kind of item, not only tasks.
- **Header bars show at most three buttons, the overflow menu counted (`crates/studyfrog-ui`, STORY-0170, EPIC-057)**: the PDF header shows Previous, Next and More… (Back to library moved to the top of More…, and zoom, hit navigation, copy, arrange and panels were already there); the stack page's Memory game moved into More actions. The item editor's header already showed two buttons. No shortcut changed. Tests: `studyfrog-ui/tests/pdf_header.rs`, `pdf_viewer.rs`, `header_core_ui.rs`, `memory_grid_ui.rs`.

### Fixed
- **Exam format mix overflow, fill-in-the-gaps blank parity and model count accounting (`crates/studyfrog-rehearsal`, `crates/studyfrog-service`, STORY-0207, BUG-0055)**: `validate_format_mix` and `resolve_format_plan` sum and scale in `u128`, so large weights never overflow; a fill-in-the-gaps question needs the blanks `[1]`..`[n]` to match its gap answers, and an empty gap answer is rejected instead of dropped; `generate_with_plan` counts `from_model` from the questions kept after balancing.
- **Reference-run script reads the whole criterion output (`scripts/bench-reference.sh`, EPIC-067, TASK-0278)**: the awk filter exited at the `time:` line while cargo still wrote "Found N outliers", so under `pipefail` a run failed at random ("no median"); the filter now reads to the end. Re-run for fsrs_schedule_100k: 5 runs spread 37.9%, extended to 10, median 28.0 ms (noisy container, powersave governor).
- **Mock exam: a part the notes cannot fill is named, and the counts follow the 40 / 30 / 30 plan where the notes allow (`crates/studyfrog-rehearsal`, `crates/studyfrog-service`, `crates/studyfrog-ui`, BUG-0055, EPIC-063)**: templates make one case per paragraph of the notes (was one per section), `curriculum_build::shortfalls` finds the parts with fewer questions than the plan asks for, and the exam preview says so for each one (`short_note`) instead of giving fewer questions silently. Tests: `studyfrog-rehearsal/tests/curriculum_build.rs` (full, partial and empty material), `studyfrog-service/src/exam_gen.rs`.
- **Note details column: priority and difficulty chips fit at 1280x800 and 960x640 (`crates/studyfrog-ui`, BUG-0056, EPIC-036)**: the column is bounded to its width, scrolls down only, and the Properties, Relations, Learning and Comments tabs wrap onto a second row; no control is removed. Checked in the running app.
- **Mock exam without a calculation part from a truncated formula note (BUG-0053, EPIC-063)**: cause confirmed, no code change; in review until the reporter confirms. A note cut to `'Standardfehler` has no formula line and makes no calculation question; the complete note gives a Part C (checked in the running app, `Formel` as the only formula note: Part C present with the complete note, absent with the cut note). The reporter has not yet confirmed.
- **A mock exam pass mark under 50 % passes the percentages above it with 4.0 (`crates/studyfrog-rehearsal`, STORY-0187, EPIC-063)**: at a 40 % pass mark, 42 % was graded 5.0 "Nicht bestanden" by the fixed bands; it is now 4.0 "Ausreichend" and passed. Test: `studyfrog-rehearsal/tests/grade.rs` (`a_pass_mark_under_fifty_passes_at_four_and_fails_below_it`).
- **Header keeps "+ Create" whole at 640 by 420, and Import moves to the Profile menu (`crates/studyfrog-ui`, BUG-0021, EPIC-065)**: in a narrow header the search field takes the room left after Create, and the first entry of the Profile menu is Import, which opens the same dialog. The search cluster moved to `view/menus_search.rs`. Tests: `studyfrog-ui/tests/window_fit_layout.rs`.
- **Rehearsal saves every step after a vault reopen (`crates/studyfrog-store`, `crates/studyfrog-service`, BUG-0041)**: the run event number is allocated from the run's stored log inside the step's transaction, so a run still on screen across a reopen no longer writes a number twice ("Could not save your progress: UNIQUE constraint failed: run_event.run_uid, run_event.seq"). Tests: `studyfrog-service/tests/run_seq.rs`.
- **A script run by hand is reported under its file name (`crates/studyfrog-ui`, BUG-0042, m12)**: the started and finished lines, the console and the log name the file typed; a job's own run keeps the job's name, its access and timeout. Test: `studyfrog-ui/tests/script_run_name.rs`.
- **`search --focus` and Ctrl+F reach the top-bar search on screens without a search field (`crates/studyfrog-ui`, BUG-0049, EPIC-064)**: when no field on screen takes the request or the shortcut, the top-bar "Search items" field takes the keyboard and the answer names it; the library query box and the PDF find field keep taking it first.
- **`state` stops naming a search field that has left the screen (`crates/studyfrog-ui`, BUG-0050, EPIC-064)**: `focused_field` is cleared when no search field has the keyboard, so after `query` and `nav home` it is null.
- **The math drill does not praise a result with a wrong working row (`crates/studyfrog-rehearsal`, `crates/studyfrog-ui`, BUG-0051, EPIC-062)**: "Correct. Well worked." only when the answer and every row are right; otherwise the box names the first wrong row, and the first-row message no longer tells the student to go back to a step before it. Test: `studyfrog-rehearsal/tests/formula_verdict.rs`.
- **A case study in an exam run shows its decision and options (`crates/studyfrog-ui`, BUG-0022, EPIC-061)**: the exam's item shows the case's decision and its options above "Your answer", with a line on how to answer; the answer is graded as before. Test: `studyfrog-ui/src/view/exam_card.rs` (`a_case_in_an_exam_shows_its_decision_and_every_option`).
- **Mock exam keyword grading counts the answers that state the notes' point (`crates/studyfrog-rehearsal`, `crates/studyfrog-agent`, `crates/studyfrog-ui`, BUG-0038, EPIC-063)**: the rubric points of questions made from the notes are the notes' own meaning, formula sides and sentences instead of generic text; a point the keyword check cannot test is shown as "Not checked by word overlap" and does not count as missing, and a question with no checked point is left out of the total. Tests: `studyfrog-rehearsal/tests/curriculum_build.rs`, `studyfrog-agent/tests/rubric_eval.rs`, `exam_grade.rs`, `studyfrog-service/tests/exam_defects.rs`.
- **A mock exam from a PDF is titled with the PDF's file name (`crates/studyfrog-ui`, BUG-0039, EPIC-063)**: the PDF list shows each file name (not the placeholder "untitled") and the exam is titled "Mock exam: <file name>". Test: `studyfrog-ui/tests/mock_exam_ui.rs` (`an_exam_from_a_pdf_is_titled_with_its_file_name`).
- **A mock exam has a countdown, warnings, auto-submit and locked answers (`crates/studyfrog-ui`, BUG-0045, EPIC-063)**: the exam shows the time left from its total duration, "Ten minutes left" and "Two minutes left" warnings, submits itself when the time runs out, and locks the answers while they are graded. Tests: `studyfrog-ui/tests/mock_exam_timer_ui.rs`.
- **A mock exam's reference answer for a card gives the card's answer alone (`crates/studyfrog-service`, BUG-0052, EPIC-063)**: the reference answer of a card or question no longer starts with its question, and a note's title is no longer repeated in its answer. Test: `studyfrog-service/tests/exam_defects.rs` (`a_cards_reference_answer_is_its_answer_alone`).
- **A rehearsal setup without a preset starts from Spaced (`crates/studyfrog-ui`, BUG-0054, m14)**: `rehearse setup` without `--preset`, and opening the rehearse screen from another screen, no longer keep the practice-only order of the previous preset, so the run writes its reviews; a practice preset named in the same setup still runs practice-only. The rehearse snapshot answers the preset (`"preset"`, `null` once a field is changed). Tests: `studyfrog-ui/tests/control_setup_default.rs`, `setup_default_form.rs`.
- **`calendar period` shows the month of its date from a fresh start (`crates/studyfrog-ui`, STORY-0124, EPIC-064)**: the month was computed as a count of months from the calendar before it had loaded, so a fresh control session landed years away; the plan now sets the month (`PlanMsg::ShowMonth`). The same plan serves `calendar session add`. Test: `studyfrog-ui/tests/control_calendar_view.rs`.
- **Telemetry rows carry their real measures (`crates/studyfrog-rehearsal`, `crates/studyfrog-service`, `crates/studyfrog-ui`, BUG-0031, BUG-0048, EPIC-060)**: a judged typed answer writes its similarity score to `review_telemetry.semantic_score`; dwell is the time from the item shown to its rating (paused time excluded), kept apart from the latency (time to the first flip or answer); `text_complexity` holds the words of the item's prompt and reference answer instead of a reading-time estimate. Tests: `studyfrog-rehearsal/tests/dwell.rs`, `studyfrog-service/tests/telemetry_fields.rs`, `telemetry_recording.rs`.
- **Memory game from the calendar and over an open run (`crates/studyfrog-ui`, BUG-0046, BUG-0047, EPIC-059)**: a calendar session row and a planned-review row offer "Memory game for <title>" beside their other actions, and the stack page's Memory game button shows the board even while a rehearsal run is open (the run waits and returns when the game is closed). Starting a run clears a finished game left on screen. Test: `studyfrog-ui/tests/memory_entry.rs`.
- **The FrogScript reference fits beside the ScriptStudio editor (`crates/studyfrog-ui`, BUG-0020, STORY-0171, STORY-0172, EPIC-058)**: the reference opens as a resizable right panel beside the editor, in a band that keeps the output pane in the window; the editor keeps a readable width, and the reference's topics, search and cookbook scroll and are clickable at 960x640, 1280x800 and 1600x900. Tests: `studyfrog-ui/tests/studio_reference_layout.rs`.
- **Calendar Timetable lists the stored lectures after a restart (`crates/studyfrog-ui`, BUG-0044, EPIC-056)**: the lectures are read with the sessions when the calendar opens (and with the homework on the homework screen), so the Timetable view no longer waits for a save; it shows what the week view shows. Test: `studyfrog-ui/tests/calendar_timetable.rs` (`stored_lectures_are_listed_after_a_restart`).
- **Case study, Memory game and Mock exam use the stack ticked in the rehearse setup (`crates/studyfrog-ui`, BUG-0023, BUG-0024, BUG-0040)**: the three buttons work on the stack ticked in the What step, and fall back to the stack open in the library only when none is ticked; the exam title names its stack and the too-few-cards message names the stack. Test: `studyfrog-ui/tests/memory_mode.rs`, `case_study_ui.rs`, `mock_exam_ui.rs`.
- **Memory time attack HUD counts down and stops at the limit (`crates/studyfrog-rehearsal`, `crates/studyfrog-ui`, BUG-0025)**: the HUD shows the time left in a 60, 90 or 120 s game, never more than the limit. Test: `studyfrog-rehearsal/tests/memory_summary.rs`.
- **Memory timeout turns the unmatched cards face up (`crates/studyfrog-rehearsal`, BUG-0026)**: when the time runs out every card not yet matched is shown, so the message matches the board. Test: `studyfrog-rehearsal/tests/memory.rs`.
- **Memory HUD time stops at the win (`crates/studyfrog-rehearsal`, BUG-0027)**: a finished game keeps its clock, so the HUD and the win message show the same time. Test: `studyfrog-rehearsal/tests/memory.rs`.
- **Memory board fits the space below its message at 960 by 640 (`crates/studyfrog-ui`, BUG-0028)**: the board is sized to the height left under the HUD, message and Play again rather than in an unbounded scroll area.
- **Rehearse setup footer wraps so the item count is not cut off (`crates/studyfrog-ui`, BUG-0029)**: at 1280 by 800 the count sits in full beside Next, and at 960 by 640 the Next button is in view.
- **Rehearsal rating buttons stay level and the hint line stays on screen (`crates/studyfrog-ui`, STORY-0195, EPIC-065)**: a previous rating no longer pushes the Good button down or cuts off the keyboard hints at 1280 by 800; the item area takes the room the rating line needs. Test: `studyfrog-ui/tests/run_previous_mark.rs`.
- **Script stderr is its own stream (`crates/studyfrog-ui`, BUG-0033, STORY-0152, EPIC-049)**: a script run in the console gives its outcome, its stdout and its stderr as separate entries; stderr shows under Stderr / Errors in the error colour and not under Stdout, and the exported log labels each line `Stdout:` or `Stderr:`. Tests: `studyfrog-ui/tests/console.rs`.
- **CSV import into the chosen stack uses that stack (`crates/studyfrog-service`, BUG-0037)**: a file whose stack column names the chosen stack puts its cards in that stack instead of a new `statistik/statistik`; other stacks still nest under the chosen one. Test: `studyfrog-service/tests/xfer_destination.rs`.
- **Header and rehearsal top strip fit the window at 640 by 420 (`crates/studyfrog-ui`, BUG-0021)**: the header keeps its sidebar toggle, logo, search and Create without drawing over each other (Import and the vault name drop out at that width), and the rehearsal top strip wraps so Comments no longer covers the position and the timer.
- **Reports tab row wraps at 960 by 640 (`crates/studyfrog-ui`, BUG-0032)**: every one of the nine tabs is reachable, the last ones on a second line.
- **Singular counts and reminder wording (`crates/studyfrog-core`, `crates/studyfrog-calendar`, `crates/studyfrog-ui`, BUG-0034)**: "1 day" for the streak, the report bucket and the session reminders, "1 page" for the PDF find and the library, "1 run" in the learn report from one shared helper (`studyfrog_core::count`); a homework reminder says the time left before the due time ("due in 1 h 8 min") in both its toast and its notification line.
- **Week view columns fit the window (`crates/studyfrog-ui`, BUG-0035)**: the seven day columns leave room for the gaps between them and the scroll bar, so Sunday and its homework deadline dot are visible at 1280 by 800; the day headers are labelled for screen readers.
- **Reminder toasts sit at the bottom right (`crates/studyfrog-ui`, BUG-0036)**: the stack moved from the top right, where it covered the Day view's "+ Add session" button, to the bottom right above the status bar.

## [1.0.0] - 2026-11-22

### Added
- **Mock exam from a lecture PDF (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0186)**: "Mock exam from a PDF" on the rehearse setup lists the vault's PDFs; the chosen PDF's text (its slides) makes the exam with the same generator as the notes, and paths outside the vault or non-PDF files are refused.
- **Mock exam says when no model is selected (`crates/studyfrog-ui`, STORY-0186)**: the exam view explains that the questions come from templates and how to select a local model; a remote model that may not see the notes is named as such.
- **Mock exam answers graded by the local model (`crates/studyfrog-agent`, `crates/studyfrog-ui`, STORY-0187)**: with a local model selected, "Finish and grade" asks it to judge each rubric point; without one, or when its reply cannot be read, the keyword check grades that answer and the reason is shown. The result says who graded.
- **Tooltip delay is a named setting and a click hides the tooltip (`crates/studyfrog-ui`, STORY-0169)**: `view::tips::DELAY_SECS` (0.5 s) is applied by the theme; a test checks that a click hides the tooltip of the control clicked.
- **Markdown card decks import (`crates/studyfrog-service`, STORY-0168)**: a `.md` file is detected by the import dialog and read as cards, one per `## ` heading (the heading is the front, the text below it the back), or as one note when it has no such heading. Headings inside code fences do not count.
- **Learn-mode research platform complete (EPIC-066, STORY-0198)**: a learn mode is drawn, edited, saved, run as practice only, and reported on its own in the app; the window check is recorded in STORY-0198.
- **Self-loops on the learn-mode canvas (`crates/studyfrog-ui`, STORY-0198)**: a wire from a step to itself is drawn as a loop above the step, with its arrow back onto it, and can be selected like other wires.
- **Board WIP limit counts every work item (`tools/repo-lint`)**: epics and stories count toward the three in progress, as tasks, subtasks and bugs already did; the board header and the gate use the same count.
- **Select and delete a wire on the learn-mode canvas (`crates/studyfrog-ui`, STORY-0198)**: a click on a wire selects it (drawn thicker); "Delete selected" removes the selected wire, or the selected step with its wires. Clicking a step or empty space clears the wire selection.
- **Window fixes from the review checks (`crates/studyfrog-ui`, STORY-0152, STORY-0150, STORY-0167)**: the console output bar fits a narrow window (its buttons wrap instead of covering the search); the script success marker is ASCII, so it no longer draws as an empty box; ScriptStudio keeps keyboard focus after a Tab completion; the homework dialog scrolls, so its Delete row stays reachable on a short window.
- **Memory game card flip and keyboard play (`crates/studyfrog-ui`, STORY-0174)**: a card turns over in 0.2 s (at once with Reduce motion), Tab reaches the cards, Enter or Space turns a face-down card over, the arrow keys move between cards, and the card with keyboard focus has a ring. The mismatch shake stops under Reduce motion.
- **Memory game launch covered by tests (`crates/studyfrog-ui`, STORY-0175)**: the rehearse setup's and the stack page's "Memory game" buttons are tested to start a game. Due-card selection is still open.
- **FrogScript reference lists the lifecycle events (`crates/studyfrog-script`, `crates/studyfrog-ui`, STORY-0171)**: the reference panel in ScriptStudio has a "Lifecycle Events" category with one entry for each event a job in `scripts.toml` can start on, with "Add example to the script" and search by name. A test fails if an event has no entry.
- **PDF page delete asks first (`crates/studyfrog-ui`, STORY-0054)**: "Delete page" in the arrange screen asks "Take this page out of the new file? Undo brings it back." with "Delete it" or "Keep it"; Undo still brings a removed page back.
- **PDF note delete asks first (`crates/studyfrog-ui`, STORY-0054)**: "Delete" on a note in the PDF viewer asks "Delete this note? It cannot be undone." with "Delete it" or "Keep it"; before, the note was deleted at once although its tooltip said it deleted after confirmation.
- **Diary entry delete asks first (`crates/studyfrog-ui`)**: "Delete entry" now asks "Delete this entry? It cannot be undone." with "Delete it" or "Keep it"; before, the entry was deleted at once although its tooltip said it deleted after confirmation.
- **Delete a homework item (`crates/studyfrog-ui`, STORY-0167)**: the edit dialog has "Delete item", which asks for confirmation before the item is removed; the dialog closes once it is gone, and a refused delete keeps it open with the reason.
- **Benchmark run-to-run spread script (`scripts/bench-spread.sh`, EPIC-067)**: runs one criterion benchmark case N times and reports its mean, coefficient of variation and spread, for setting the noise floor of the regression rule on a reference machine.
- **Benchmark reference runs with machine identity (`scripts/bench-machine.sh`, `scripts/bench-reference.sh`, EPIC-067, STORY-0201, TASK-0277, TASK-0278)**: `bench-reference.sh <label> <out>` runs each budgeted benchmark case 5 times as separate processes and writes one median per run (ns) to `<out>/<case>.ns`, extending a case that spreads more than 5% to 10 runs; `bench-machine.sh` records the CPU, kernel, rustc, commit, governor, boost, SMT and load of the run, and warns about hygiene items not met. Interleaving two builds is still open (TASK-0279).
- **Benchmark regression rule (`repo-lint bench-compare`, `scripts/bench-compare.sh`, EPIC-067, STORY-0202, TASK-0280, TASK-0281)**: compares two runs and flags a regression only when the bootstrap 95% interval of the ratio of medians excludes 1.0 and the slowdown exceeds max(5%, 3 times the baseline's spread); exit 1 only for a regression. The gate now also runs the repo-lint tests (`scripts/check.sh`).
- **FrogScript keyboard completion and cursor placement (`crates/studyfrog-script`, `crates/studyfrog-ui`, STORY-0150)**: in ScriptStudio, Tab accepts the highlighted name, Up and Down move the highlight (wrapping), Esc hides the list until the word changes, and the cursor goes to the end of an inserted name.
- **Custom runs order their returning items (`crates/studyfrog-rehearsal`, `crates/studyfrog-service`, STORY-0198)**: an item a custom mode sends back comes back after 2 (Again), 4 (Hard), 6 (Good) or 8 (Easy) other items, not at the end of the run. Built-in runs are unchanged.
- **Custom learn-mode report, opt-in (`crates/studyfrog-store`, `crates/studyfrog-ui`, STORY-0198)**: "Show report" in the learn-mode editor reads one mode's custom runs (runs, finished, items, entries per step, ratings, branches); built-in reports never include them.
- **Learn-mode canvas (`crates/studyfrog-ui`, `crates/studyfrog-service`, STORY-0198)**: the learn mode is drawn in the editor: steps are boxes you can place and drag, wires are arrows drawn between them in connect mode, and the places are saved with the mode.
- **A custom learn mode decides what comes back (`crates/studyfrog-rehearsal`, `crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0198)**: in a custom run an item is shown again only when its graph sends it back to a rate node; a stop or a loop at its cap takes it out of the run. Built-in runs are unchanged.
- **Start a custom learn-mode run from the app (`crates/studyfrog-ui`, STORY-0198)**: "Start a run" in the learn-mode editor runs the saved mode as a practice-only run; its steps are logged apart from the built-in modes.
- **Learn-mode editor (`crates/studyfrog-ui`, STORY-0198)**: "Learn modes" on the Rehearse setup opens a form to list, name, build, check and save a learn mode (nodes, wires, branch conditions); a mode that cannot run is not saved, and the reasons are listed.
- **Custom learn-mode runs (`crates/studyfrog-store`, `crates/studyfrog-service`, `crates/studyfrog-rehearsal`, STORY-0198, part 2 backend)**: a checked learn mode runs practice-only (no reviews, no schedule change); each rating walks its graph and the path is stored with a snapshot of the mode; built-in history and counts leave custom runs out. The app's start, the editor and the custom report are still to come.
- **Learn-mode graphs and their storage (`crates/studyfrog-rehearsal`, `crates/studyfrog-service`, STORY-0198, part 1)**: a learn mode is a graph of start, present, recall, rate, schedule, log, branch and stop nodes; a graph that cannot run is refused with the reasons (no start, a loop without a stop, an unknown node, and others); valid graphs are saved as TOML in the vault's `learn_modes` folder. The canvas editor and running a custom mode are still to come.
- **Previous rating on the rating buttons (`crates/studyfrog-ui`, STORY-0196)**: the button of the item's previous rating shows "Last time: Difficult / Medium / Easy" in red, orange or green; an item with no previous rating shows nothing.
- **Rehearsal item takes more of the screen (`crates/studyfrog-ui`, STORY-0195)**: the card spans the window, the item's text scrolls in its own area, and the rating buttons stay under it; a `--screenshot run-long` demo checks it at 1280 by 800, 1600 by 1000 and the 640 by 420 minimum.
- **Calendar and reminders in the control protocol (`crates/studyfrog-ui`, `crates/studyfrog-service`, STORY-0194)**: `calendar view`, `calendar period DATE` and `calendar homework DATE` drive the calendar screen, and the homework answer gives each item's urgency; `reminders due` answers the calendar and homework reminders due now without marking them shown.
- **Search focus and tooltips in the control protocol (`crates/studyfrog-ui`, STORY-0192)**: `search --focus` gives the library or PDF search field the keyboard, and the answer names the field once it has it; `help tooltip NAME` answers a control's explanation and shortcut; the state names the focused field.
- **FrogScript editor help (`crates/studyfrog-script`, `crates/studyfrog-ui`, STORY-0150)**: the ScriptStudio text is coloured by token, the bracket at the cursor is matched, the first error is underlined and named, and the names that complete the word before the cursor are offered. The research on embedding a DSL is in `docs/research/scripting-dsl-evaluation.md`.
- **Scheduled and event-bound jobs in the app (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0151)**: the Console's Jobs editor lists the jobs in `scripts.toml` with their status and next run, switches them on or off, edits their schedule and events, adds a job, runs one now or a script with arguments, shows their recent runs, and saves the file only when it checks.
- **Console output: times, search and export (`crates/studyfrog-ui`, STORY-0152)**: each console line shows its time on the app's clock; the output can be searched by text; "Export log" writes the transcript to `logs/script_runs.log` in the vault.
- **Import destinations, PDF cards and PDF find in the control protocol (`crates/studyfrog-ui`, `crates/studyfrog-service`, STORY-0191)**: `import --root` and `import --new-stack NAME` choose the import destination as the dialog does; the import answer lists the pages left out; PDF import is covered by tests (one card per page with text, the others listed); `pdf find TEXT [--file F]` opens the PDF and sets its find field, and the answer lists the pages found.
- **Writing and grading a mock exam (`crates/studyfrog-ui`, STORY-0187)**: the generated exam can be started and answered question by question; finishing grades each answer against its rubric and shows the grade (German scale, pass at 50 %), the points by topic, and the review of every question.
- **Mock exam from notes in the app (`crates/studyfrog-ui`, STORY-0186)**: "Mock exam" on the rehearse setup makes an exam of ten questions from the stack's notes, with the local model when one may see them, and saves it into the stack as a note with its reference answers.
- **Mock exam generation from notes (`crates/studyfrog-rehearsal`, `crates/studyfrog-agent`, `crates/studyfrog-service`, STORY-0186)**: the notes are cut into topic chunks; the local model writes questions per chunk (or templates stand in); each question is checked against its topic's notes; the result is balanced to 40/30/30 and assembled into a mock exam with reference answers.
- **Math drill (`crates/studyfrog-ui`, STORY-0184)**: a standard-error problem with given values, optional working rows and a keypad; the check names the first wrong row and its likely mistake; the full solution is shown on demand.
- **Case studies in a rehearsal run (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0199)**: a case question in a run shows its options and justification, a check against the rubric, and the result under the revealed answer, with the suggested rating filled in. The rating stays the student's; the check's score is stored like a typed answer's score.
- **Local-model scoring of case justifications (`crates/studyfrog-agent`, `crates/studyfrog-ui`, STORY-0200)**: a selected local model checks a case justification against the rubric; a remote one is never sent the text. The keyword check shows at once and is replaced when the model answers; caveats are shown.
- **Tour offer as a card on Home (`crates/studyfrog-ui`)**: the first-run tour is offered on the Home screen with "Start the tour" and "Not now", instead of a toast over the screen.
- **Case studies (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0179, STORY-0181)**: a case study is a question file with its decision, options, expected choices and rubric in front matter; the rehearse screen's "Case study" opens a stack's cases, scores a choice and a justification against the rubric, and shows the result as pills.
- **Switch for the calibrated starting difficulty (`crates/studyfrog-config`, `crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0177)**: Settings, Scheduling, has a checkbox (on by default) that writes `calibrated_start` under `[srs]` in `studyfrog.toml`.
- **Calibrated starting difficulty (`crates/studyfrog-service`, `crates/studyfrog-store`, STORY-0177)**: a new item's first FSRS review starts from the scheduler's prior blended with the user's earlier first-answer difficulties. On by default; unchanged until the user has timed answers.
- **Welcome and notification toasts at the top right (`crates/studyfrog-ui`)**: they sit under the header, 340 points wide, so they no longer cover the setup's Next button or the step bar.
- **Consolidation chart on the Cognitive profile tab (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0177)**: the mean days between passing reviews, by week, from the review history.
- **Memory game records and victory dialog (`crates/studyfrog-config`, `crates/studyfrog-ui`, STORY-0175)**: the best time of each stack is saved in `app.toml` and survives restarts; a completed game opens a victory dialog with the summary and the personal-best line.
- **Memory game timing, summary and entry points (`crates/studyfrog-ui`, STORY-0175)**: Stopwatch or a 60, 90 or 120 second countdown; the HUD time changes colour as the countdown runs out; a victory summary (time, moves, accuracy, speed, bonus, best streak, personal best for this session) or the timeout line; "Memory game" on the stack page.
- **Telemetry recording on timed ratings (`crates/studyfrog-service`, STORY-0176)**: every rated answer with timing writes one telemetry row; untimed ratings write none.
- **Memory game screen fixes (`crates/studyfrog-ui`, STORY-0174)**: the close button sits in the HUD row, so it no longer needs scrolling.
- **Cognitive profile tab on the Reports screen (`crates/studyfrog-service`, `crates/studyfrog-ui`, STORY-0178)**: latency against word length, weekly passing share, learning velocity and a metacognitive calibration widget, all from the review history. Telemetry is not recorded yet, so semantic scores and stability are not shown.
- **Memory mode game on the rehearse screen (`crates/studyfrog-ui`, STORY-0174)**: a "Memory game" button in the setup starts a game from the open stack. Cards lay out in 2 to 8 columns at the largest size that fits; the board has dot-patterned backs, a match glow, a shaking mismatch and a HUD with time, moves, pairs left and streak.
- **Homework model, storage and reminders (`crates/studyfrog-calendar`, `crates/studyfrog-store`, `crates/studyfrog-service`)**:
  - Pure homework item model with title, course, start and due date and time, estimate, priority, status, checklist, and notes, with validation and lifecycle helpers (STORY-0165, EPIC-056).
  - SQLite storage in schema step 29 (`homework_item`, `homework_step`, `homework_fired`) and a deterministic TOML form, mirrored to `.studyfrog/homework.toml` on every change (STORY-0165).
  - Lead-time reminders at 24 hours, 2 hours and 15 minutes before a deadline, each shown once and remembered across restarts (STORY-0166).
- **Memory Mode engine (`crates/studyfrog-rehearsal`)**: deck pairing of each card's front and back, a seeded deterministic shuffle, the two-turn flip cycle with mismatch display delay, move and streak counters, and a time limit (STORY-0173, EPIC-059).
- **Mock exam model (`crates/studyfrog-rehearsal`)**: weighted sections and questions with validation, navigation with flags and an unanswered-question review, and a strict timer with 10-minute and 2-minute alerts and expiry (STORY-0185, EPIC-063).
- **Psychometric formula engine and step diagnosis (`crates/studyfrog-rehearsal`)**: standard error of measurement, confidence intervals, standard scores, Spearman-Brown, and z-values; a seeded exercise generator; a three-step trace of the standard error with a diagnosis that names the first wrong step and the likely mistake (STORY-0182, STORY-0183, EPIC-062).
- **Cognitive telemetry storage (`crates/studyfrog-store`)**: `review_telemetry` in schema step 30 with latency, dwell, semantic score and text size per answer, loaded by item list or time range (STORY-0176, EPIC-060).
- **Difficulty and velocity engine (`crates/studyfrog-report`)**: empirical item difficulty on a 1 to 10 scale, the learning-velocity slope, and a prior calibrated from telemetry (STORY-0177, EPIC-060).
- **Memory Mode summaries (`crates/studyfrog-rehearsal`)**: time-attack options, the danger level as the countdown runs out, and the victory summary with speed, bonus and personal best (STORY-0175, EPIC-059).
- **Case-study model (`crates/studyfrog-rehearsal`)**: vignette, decision, options, expected choices and rubric, with validation (STORY-0179, EPIC-061).
- **Rubric evaluator (`crates/studyfrog-agent`)**: parses the local model's JSON, recomputes the score (40 % choice, 60 % rubric), and falls back to a keyword check when no model runs (STORY-0180, EPIC-061).
- **Homework reminders in the app (`crates/studyfrog-ui`)**: homework deadlines are checked on the same schedule as calendar reminders and shown as a toast and a notification line (STORY-0166, EPIC-056).
- **FrogScript cookbook (`studyfrog-script`, `crates/studyfrog-ui`)**: ten recipes (seven read-only) at the foot of the reference; each can be inserted into the editor, and read-only ones run in a new console tab (STORY-0172, EPIC-058).
- **FrogScript reference in ScriptStudio (`studyfrog-script`, `crates/studyfrog-ui`)**: a categorised reference of the language's syntax, operators, control flow, built-in functions and vault objects, searchable, beside the editor; the example of a topic can be added to the script (STORY-0171, EPIC-058).
- **Homework in the control protocol (`crates/studyfrog-ui`)**: `homework list`, `homework tab`, `homework new` (with due, time, estimate, priority, stack and checklist flags), `homework advance` and `homework tick` drive the homework screen from the command line; a refused change answers with its reason (STORY-0190, EPIC-064).
- **Search shortcut (`crates/studyfrog-ui`)**: Ctrl+F (and `/` when nothing has the keyboard) focuses and selects the search field on the library and in the PDF viewer, and the field names the shortcut (STORY-0188, EPIC-057).
- **Less crowded headers and roomier spacing (`crates/studyfrog-ui`)**: the stack page, item list, item editor and PDF viewer move low-frequency buttons into overflow menus; the Comfortable and Spacious densities are 25% and 50% wider; toasts sit at the bottom right, the sidebar stops above the floating menu button, and the item editor's details scroll sideways (STORY-0170, EPIC-057).
- **Hover explanations for every text button (`crates/studyfrog-ui`)**: each text button in the app explains itself on hover, and a source check fails when a new one does not (STORY-0189, EPIC-057).
- **Guard for text buttons without hover explanations (`crates/studyfrog-ui`)**: a source scan counts the buttons without one; the count may only go down (STORY-0189, EPIC-057).
- **Tooltips with shortcuts (`crates/studyfrog-ui`)**: every icon button explains itself on hover, and the rating buttons, difficulty marks, Show answer and Skip show their keyboard shortcut (STORY-0169, EPIC-057).
- **PDF import as cards (`studyfrog-service`)**: a PDF is read as one card per page with text, the page's first line as the front and the rest as the back; scans and single-line pages are reported in the preview (STORY-0168, EPIC-057).
- **Import destinations (`crates/studyfrog-ui`, `studyfrog-service`)**: the Import tab can put items as the file says, at the vault root, into an existing stack, or into a new stack named in a field; an empty target is the vault root; PDFs are pointed at Add PDF (STORY-0168, EPIC-057).
- **One Import entry point (`crates/studyfrog-ui`, `studyfrog-service`)**: the items toolbar, stack menu and More menu each offer a single Import (the transfer dialog) and Export; extension importers moved to "Add from extension…"; the Import tab says which importer a chosen path looks like (STORY-0168, EPIC-057).
- **Homework and calendar reminders share one engine (`studyfrog-calendar`, `studyfrog-service`, `studyfrog-ui`)**: `reminders::due_all` returns calendar and homework reminders together; `take_due_reminders` marks both as shown, and the app checks them with one effect (STORY-0166, EPIC-056).
- **Homework planner screen (`crates/studyfrog-ui`)**: a Homework screen in the Plan group with All, Active, Due today, Overdue and Completed tabs, a countdown and urgency on each item, checklist ticks, Start / Complete / Reopen, and a dialog to create and edit items with stack, due date and time, estimate, priority, checklist and notes (STORY-0167, EPIC-056).
- **Homework on the calendar (`crates/studyfrog-ui`, `studyfrog-calendar`)**: homework deadlines appear as urgency-coloured dots on the month and week grids and as a "Homework due" card in the day view; `due_on` and `most_urgent` in `homework_query` pick the day's items and their most pressing urgency (STORY-0166, EPIC-056).
- **German grade scale (`crates/studyfrog-rehearsal`)**: percentage and points summation, the eleven-band scale from 1.0 to 5.0, and a pass mark that forces a fail below it (STORY-0187, EPIC-063).

### Fixed
- **Exam answer marks are ASCII (`crates/studyfrog-ui`, STORY-0128)**: the correct answer is marked "+", a wrong pick "x", the selected choice "(*)" and the others "( )", because the UI font has no ✓, ✗, ● or ○; the streaming assistant cursor and the chat error label are ASCII too.
- **Learn-mode custom report reads grammatically (`crates/studyfrog-ui`, STORY-0198)**: "1 run" and "1 item reached" use the singular; counts of 2 or more and 0 keep the plural.
- **Sidecar port test no longer flakes (`crates/studyfrog-agent`, TASK-0271)**: the tests that probe the default port range run one at a time, so a probe in one test cannot bind the port another test is checking.
- **Faster first ratings in large vaults (`crates/studyfrog-store`, `crates/studyfrog-service`, STORY-0177)**: a new card's first rating no longer reads the whole vault's overview for its deadlines (about 640 ms to about 6 ms at 100,000 cards, debug build). The starting-difficulty lookup is bounded and stays about 5 ms at a million reviews.
- **Rerating keeps the calibrated start (`crates/studyfrog-service`, STORY-0177)**: replaying a first review uses the calibration in force when it was made, so rerating to the same rating leaves the schedule unchanged.

## [0.9.0] - 2026-11-16

### Added
- **FrogCanvas 2D Spatial Mind-Map & Concept Board (`crates/studyfrog-graph`, `crates/studyfrog-ui`)**:
  - Infinite 2D spatial learning board where cards, notes, text labels, and visual excerpts can be arranged, clustered, and connected (EPIC-050).
  - TOML document serialization and data model (`_canvas.toml` in stack directories) for `CanvasDoc`, `CanvasNode`, `CanvasEdge`, and `CanvasFrame` (TASK-0253, STORY-0153).
  - 2D spatial quadtree index with recursive quadrant subdivision (NW, NE, SW, SE) and sub-millisecond bounding box viewport culling maintaining a steady 60 FPS during pan and zoom (TASK-0254, STORY-0153).
  - Interactive canvas view with smooth 2D affine matrix panning, zooming clamped between 0.05x and 20.0x, and interactive node dragging (TASK-0255, STORY-0154).
  - Directed and bidirectional cubic Bézier curve edge rendering with relationship labels, geometric connection anchors, and card preview popovers upon node selection or hover (TASK-0256, STORY-0154).
- **PDF Precision Area Cropping & 1-Click Split Navigation (`crates/studyfrog-pdf`, `crates/studyfrog-ui`)**:
  - Rectangular drag-selection lasso tool in the PDF viewer to crop visual diagrams and equations directly into flashcards and notes (TASK-0257, STORY-0155, EPIC-051).
  - Frontmatter coordinate anchor URI schema (`source: path/to/doc.pdf#page=N&bbox=x,y,w,h`) linking visual excerpts to precise document coordinates (TASK-0257, STORY-0155).
  - High-performance bounded LRU memory and disk rasterization cache (`CropLruCache`) for fast card image rendering without full-page re-renders (TASK-0258, STORY-0155).
  - 1-click split-screen "PDF Jump" contextual navigation button rendered on rehearsal cards and notes containing coordinate anchors (TASK-0259, STORY-0156).
  - Synchronized split-pane PDF viewer positioning to target page with flashing rectangular highlight overlay on coordinates and adjustable split ratio from 0.2 to 0.8 (TASK-0260, STORY-0156).
- **Continuous Priority Queue & Anti-Paralysis Slicing (`crates/studyfrog-srs`, `crates/studyfrog-service`)**:
  - Effective priority scoring function blending normalized user priority (40%), retrievability shortfall `1 - R(t)` (35%), and familiarity shortfall `1 - F(c)` (25%) into an urgency score in `[0.0, 1.0]` (TASK-0261, STORY-0157, EPIC-052).
  - Priority-ordered queue builder slicing daily overdue queues to user-configured session budgets (e.g. 30 cards), guaranteeing high-priority card review and preventing cognitive backlog paralysis (TASK-0262, STORY-0157, EPIC-052).
- **Gamified & Multi-Modal Active Recall: Match Mode & Distractor Exams (`crates/studyfrog-rehearsal`, `crates/studyfrog-candle`, `crates/studyfrog-service`, `crates/studyfrog-ui`)**:
  - Interactive Match Mode: 3x4 tile pairing drills (6 pairs / 12 tiles) with PRNG shuffling, monotonic millisecond timing, mismatch time penalties, and consecutive match streak multipliers (TASK-0263, STORY-0158, EPIC-053).
  - Responsive match grid UI in egui with tile selection highlighting, match/mismatch visual feedback animations, and score summary reports (TASK-0264, STORY-0158, EPIC-053).
  - Semantic distractor generation: extracts plausible multiple-choice distractors from active stack/vault card answers using MiniLM embedding cosine nearest neighbors rather than random shuffling (TASK-0265, STORY-0159, EPIC-053).
  - Distractor exam view: interactive multiple-choice question exam drills with automated grading, instant feedback, and integration into academic grading bands (TASK-0266, STORY-0159, EPIC-053).
- **Algorithmic Optimization & Frictionless Capture: FSRS Auto-Tuning & Inline Cards (`crates/studyfrog-srs`, `crates/studyfrog-md`, `crates/studyfrog-store`, `crates/studyfrog-service`, `crates/studyfrog-ui`)**:
  - Local Adam gradient descent optimizer tuning all 21 FSRS-6 parameters on SQLite historical review logs via binary cross-entropy log-loss minimization and L2 regularization shrinking toward published priors (TASK-0267, STORY-0160, EPIC-054).
  - Rehearsal Settings UI dashboard with "Optimize Parameters" action, RMSE evaluation metrics, and log-loss improvement tracking before applying weights (TASK-0268, STORY-0160, EPIC-054).
  - Inline card delimiter scanner parsing `Term :: Definition` (forward) and `Term ::: Definition` (reversible) pairs from markdown note bodies while cleanly ignoring colons in code fences, backticks, math blocks, and tables (TASK-0269, STORY-0161, EPIC-054).
  - SQLite virtual item indexing syncing inline cards directly into the vault index with line coordinate anchors (`source: note#L<line>`) without cluttering filesystem directories (TASK-0270, STORY-0161, EPIC-054).
- **Offline Local SLM Internal Assistant (`crates/studyfrog-agent`, `crates/studyfrog-net`, `crates/studyfrog-store`, `crates/studyfrog-service`, `crates/studyfrog-ui`)**:
  - Managed background sidecar supervisor running local `llama-server` bound strictly to `127.0.0.1` on ephemeral ports (18000..=19000) with stdin watchdog termination preventing zombie processes (TASK-0271, STORY-0162, EPIC-055).
  - Automatic 5-minute idle shutdown supervisor stopping child processes after 300s of inactivity to achieve 100% RAM reclamation (0 MB idle footprint) with seamless on-demand reactivation (TASK-0272, STORY-0162, EPIC-055).
  - Curated GGUF model catalog featuring `Qwen2.5-1.5B-Instruct` (default, Apache 2.0), `Llama-3.2-1B-Instruct`, and `SmolLM2-360M-Instruct`, with chunked range-resuming downloader and SHA-256 integrity verification (TASK-0273, STORY-0163, EPIC-055).
  - Server-Sent Events (SSE) streaming client and real-time chat UI with pulsing "Thinking / Analyzing vault..." indicator during prefill transitioning into live token streaming (TASK-0274, STORY-0163, EPIC-055).
  - Local SQLite `vault_embeddings` table storing 384-dimensional dense vectors with sub-15ms cosine similarity ranking over note chunk embeddings for on-device RAG (TASK-0275, STORY-0164, EPIC-055).
  - Strict 1024-token prompt budgeting (system <= 150, history <= 150, context <= 768 tokens) limiting CPU prefill latency to under 3.5 seconds on consumer hardware (TASK-0276, STORY-0164, EPIC-055).
- **Multilingual Prompt Library & German Translations (`docs/prompts/de/*`, `crates/studyfrog-service`)**:
  - Comprehensive German localized translations of the standard prompt library: Exam (`exam.md`), Flashcards (`flashcards.md`), Notes (`notes.md`), Questions (`questions.md`), Socratic dialogue (`socratic.md`), and Summary (`summary.md`).
  - Prompt language selector allowing toggling between English and German prompt templates.
- **Universal Transfer & Import Overhaul (`crates/studyfrog-ui`)**:
  - Restructured modal transfer and import dialogs (`modal_transfer_import.rs`, `modal_transfer_preview.rs`, `modal_import.rs`) with multi-format detection, stack bulk operations, and live previews.

### Changed
- Metacognitive calibration scoring refined with difficulty weighting to avoid unwarranted overconfidence penalties on intrinsically difficult cards (`crates/studyfrog-report/src/calibration.rs`).
- Assistant chat UI enhanced with real-time token streaming and thinking indicator animations (`crates/studyfrog-ui/src/view/agent_chat_stream.rs`).

### Fixed
- Sidecar process lifetime safety: prevented orphan child process accumulation by implementing cross-platform parent-to-child stdin EOF pipe watchdogs (`crates/studyfrog-agent/src/sidecar_process.rs`).
- Priority scoring numerical stability: NaN and non-finite inputs to priority, retrievability, or familiarity are safely handled and clamped within `[0.0, 1.0]` (`crates/studyfrog-srs/src/priority_slice.rs`).
- Markdown card parser collisions: prevented standard Markdown colons, C++/Rust namespaces (`std::`), code fences, and math blocks from triggering false inline card extractions (`crates/studyfrog-md/src/inline_cards.rs`).

## [0.8.0] - 2026-11-11

### Added
- **FrogScript Domain-Specific Language (`crates/studyfrog-script`)**:
  - Embedded zero-dependency, pure Rust domain-specific language engine for study automation, active-recall queue steering, deck generation, and custom queries.
  - Python-like syntax featuring first-class pipeline operator (`|`), variable assignment, semicolon line chaining (`;`), and duration literals (`30s`, `15m`, `2h`, `7d`).
  - Query verbs (`from stack "..."`, `where tag "..."`, `set priority "..."`) with predicate matchers (`tag`, `type`, `difficulty`, `priority`).
  - First-class closures (`{ item -> ... }`) for inline card and item transformations.
  - Native runtime globals and stdlib: `print`, `log_info`, `log_warn`, `log_error`, `len`, `now`, `today`, `days_ago`.
- **ScriptStudio Multi-Tab Workbench (`crates/studyfrog-ui`)**:
  - Multi-tab script editor supporting concurrent open tabs, draft editing, tab switching, and closing.
  - File persistence: discover, open, edit, and save `.frog`, `.py`, and `.sh` scripts directly in `.studyfrog/scripts/` (vault level) or application scripts folder (`<app home>/scripts/`).
  - In-app execution bridge: executes FrogScript against active vault items in memory with live stdout/stderr capture and lifecycle notifications.
- **Split Console Output Streams & Filtering**:
  - Independent stdout and stderr execution capture buffers.
  - Stream-level filter controls (`All`, `Stdout`, `Stderr / Errors`, `Events`) for isolating execution output from errors or lifecycle events.
- **Expanded Application Lifecycle Event Bus**:
  - Expanded from 6 events to 14 reactive lifecycle events: `vault_opened`, `vault_closed`, `item_created`, `item_saved`, `item_deleted`, `rehearsal_started`, `rehearsal_stepped`, `rehearsal_finished`, `exam_finished`, `diary_entry_saved`, `import_finished`, `backup_created`, `focus_block_finished`, `script_finished`.

## [0.7.1] - 2026-11-11

### Added
- Assistant conversational chat interface: dedicated "Chat" tab on the Assistant screen with conversational message flow, auto-scrolling message bubbles, real-time message sending on Enter or Send button click, clear conversation action, and full integration with configured local and remote LLM providers.
- Universal importer routing in items and stack views: the "Import…" button in Library items and stack views now routes directly to the universal importer, enabling import of notes, stacks, questions, flashcards, CSV/TSV spreadsheets, Obsidian vaults, and Anki packages.
- Folders viewer document launching: double-clicking or right-clicking "Open" on PDF documents in the Folders tree navigator directly opens them in the built-in PDF viewer.
- Dedicated item deletion modal: replaced the inline deletion confirmation bar with a centered modal popup with backdrop dimming and explicit Delete Item and Cancel buttons.
- Item difficulty-weighted metacognitive calibration: self-assessment calibration scoring now incorporates item difficulty to prevent false overconfidence penalties on difficult materials.
- Window icon integration: embedded stylized *Breviceps macrops* vector logo icon into application window and desktop packaging.
- ScriptStudio & FrogScript rebranding and language specification (STORY-0149, STORY-0150, EPIC-049): rebranded ScriptRunner console to ScriptStudio to respect external trademarks, and authored the FrogScript DSL language specification (v1.0).

### Fixed
- Due items alignment: aligned day-end timestamp cutoffs so items scheduled for today are counted consistently across both the Dashboard and the Calendar.
- Calendar rehearsal setup item count: fixed review queue admission when initiating rehearsal from calendar and timetable cards, resolving 0-item selection and adding recursive substack inclusion.

## [0.7.0] - 2026-11-10

### Added
- Timetable schedule planner (STORY-0141, EPIC-044): weekly recurring course, lecture, seminar, and lesson scheduling with course title, day of week, start/end time, location/room, instructor, and linked stack; projects automatically into calendar events and notifications across the semester term.
- Calendar daily and weekly views (STORY-0142, EPIC-044): segmented view mode switcher (`Month | Week | Day | Timetable`); 7-day hourly column grid (06:00 to 22:00) with event chips and day selection in Week view; single-day hourly agenda with detailed session cards and review workload banner in Day view.
- Markdown macro library expansion & escaping syntax (STORY-0143, STORY-0144, EPIC-045): comprehensive macro expansions (`{rule}`, `{header:level}`, `{arrow_right}`, `{arrow_left}`, `{graph:...}`, `{hide}`, `{panel}`, `{link}`, `{bold:"..."}`, `{italic:"..."}`); escaping syntax `{#...}` to display macro tokens literally without expansion.
- Markdown list syntax enhancements (STORY-0145, EPIC-046): support for dash (`-`), asterisk (`*`), decimal numbers (`1.`, `2.`), and alphabetical markers (`a)`, `b.`, `c.`) with consistent indentation and typography.
- Stylized *Breviceps macrops* brand mascot icon (STORY-0146, EPIC-047): custom vector-drawn desert rain frog with spherical silhouette, large forward-facing eyes, and studious expression matching OKLCH palettes in both light and dark modes.
- Floating burger navigation research and prototype (STORY-0147, EPIC-047): comprehensive UX research report (`docs/research/floating-burger-navigation.md`) and collapsible floating action button prototype for instant 1-click screen access.
- Console expansion to ScriptRunner environment and internal Python API (STORY-0148, EPIC-048): multi-line code editor with recipe template picker, stdout/stderr execution stream, and complete internal `studyfrog` Python API architecture specification (`docs/research/scriptrunner-and-internal-api.md`).
- Profile and vault switchers overhaul (STORY-0138, EPIC-043): elevated card layouts, active status badges, path disambiguation, profile avatars, and safe deletion workflows.
- File navigator and in-app file dialogue overhaul (STORY-0139, EPIC-043): tree indent guides, file-type icons, inline real-time filtering, path breadcrumbs, and shortcut bookmarks.
- Expandable comments drawer in rehearsal and exam modes (STORY-0140, EPIC-043): slide-out drawer or panel to view and append item comments during active rehearsal/exam runs.

### Fixed
- Markdown viewer bold text visibility (BUG-0017): distinct typographic contrast for bold words across dark and light palettes.
- Code block indentation preservation (BUG-0018): whitespace and tab stops preserved faithfully within code blocks in markdown viewer.
- Notes stack membership made optional (BUG-0019): standalone notes permitted at vault root without requiring stack assignment.

## [0.6.1] - 2026-11-10

### Added
- In-rehearsal quick item editing: edit cards, questions, and notes without leaving or resetting the active rehearsal run; triggered via `Ctrl+E` / `Cmd+E` / `F2` shortcut or `✏ Edit` buttons on the card and top strip; automatically pauses run and timer; allows editing Title, Prompt/Front, Answer/Back, and Tags; on save, persists changes to disk, re-indexes, and hot-reloads card content in place.
- Intermittent typed active recall sampling (STORY-0136, EPIC-040): rehearsal setup introduces a configurable `Typed answer sampling (0-100%)` setting; during rehearsal, runs dynamically alternate between typed answers and flip cards according to the sampling percentage; notes are excluded from typed answer requirements; card header displays mode indicator.
- Item comments system (STORY-0135, EPIC-039): dedicated Comments tab in the Item Details notebook with author badges, timestamped entries, and add/delete controls; domain models in `studyfrog-core::comment`.
- Interactive parameter-guided content generation prompt (STORY-0137, EPIC-042): new `interactive_extract` prompt in Prompt Library under Create Content, guiding AI models through an intake interview (scope outline, language, difficulty tier, format style) before generating StudyFrog-compatible CSV cards.

### Changed
- System-driven modularity and decoupled background services (EPIC-041): file surveys, folder scans, and library index updates trigger immediately upon vault open, vault switch, and dashboard/library navigation rather than lazily waiting for the Files screen to open.
- Rehearsal lapse queue positioning: items rated `Again` during rehearsal are automatically moved to the last position of the active run queue instead of reappearing prematurely, allowing the user to complete the current cycle before re-testing lapses.

### Fixed
- Metacognitive calibration measured accuracy baseline (BUG-0016): replaced naive binary recall ratio (`recalled / total`) with a multi-factor recall accuracy baseline that weights item familiarity and novelty ($0.35 + 0.65 \times \text{familiarity}$), rating grades (`Easy` 1.0, `Good` 0.85, `Hard` 0.50, `Again` 0.0), and reading latency relative to expected reading speed; eliminates distorted 100% measured accuracy on unpracticed items.

## [0.6.0] - 2026-11-09

### Added
- Reports as CSV (STORY-0098): the totals of learned and mastered items and the active streak, and per stack its items, due, learned, mastered, reviews, average interval and retention; "Save CSV to..." next to "Export as CSV" writes into a folder chosen in the folder dialog.
- Settings > Data > **Open logs folder** shows the logs in the system's file manager (STORY-0122); the application log now also records failed background work, import and export outcomes with their warnings and skipped rows, and work that took two seconds or more (source `worker`), from the window and from headless instances.
- `studyfrog --screenshot ... --screen achievements:<folder>|global` and `--screen prompts:<category>` open the achievements filtered and the Prompts screen on a category (STORY-0087, STORY-0092).
- "Test connection to <name>" on the assistant's Connections tab (STORY-0083, STORY-0084): one tiny request without vault text through the usual path (a remote provider must be allowed first) and "Works: ..." or the error sentence under the connection, never with the key; tested against fake local servers in the Gemini and DeepSeek formats through the real HTTP sender.
- PDF viewer measurements (STORY-0085, BUG-0010): `tests/pdf_frame_time.rs` drives a 500-page file frame by frame (update, layout and tessellation; median 8.3 ms in a debug build against 16.7 ms) and counts the thumbnail requests; `tests/pdf_sharpness.rs` checks the texels per screen pixel of the painted page at every zoom, fit mode, density and window size.
- `studyfrog --screenshot ... --screen "pdf:<path>[@fit-width|fit-page|<percent>][+pages]"` opens the PDF viewer for the picture and waits for its pages to be drawn (BUG-0010).
- Scripting documentation and examples (STORY-0134): help page "Scripts, the Python API and the Console" (`scripting`, opened by F1 on the Console) and FAQ entries; spec `docs/specs/scripting.md`; `examples/scripts/` with a daily vault health check (optionally saved as a note), a weekly export that keeps the newest N, an event-bound script that tags new items, a glossary importer and a sample `scripts.toml`, each run by `cargo test -p studyfrog-app --test script_examples`; `docs/EXTENDING.md` compares compiled-in extensions with script plugins and says what scripts cannot do.
- Console screen (STORY-0133): **Console** in the sidebar takes control commands (`state`, `query ...`, `item create ...`, `logs`, `help`) and `script run JOB|FILE [ARGS]` / `script list`, sends them to the window's own control protocol (the one scripts use) or to its script runner, and shows each command with its answer, errors, lifecycle events and script output as they happen; Up/Down walk the history, Clear empties it. `studyfrog --screenshot ... --screen console`, `nav console`.
- Script runner (STORY-0132): `<app home>/scripts/scripts.toml` lists jobs (`script`, `args`, a cron `schedule` such as `30 7 * * mon-fri` or `@daily`, lifecycle events in `on`, `timeout`, `access = "read"|"write"`, `allow_destructive`, `enabled`); every window and headless instance runs the scheduled and event-bound ones as child processes with a timeout, each with a control socket of its own limited to its access, and logs how each run went (source `script`, with its output). The Python client is written to `scripts/lib/` and put on `PYTHONPATH`. New commands: `studyfrog script run FILE|JOB [--access] [--timeout] [-- ARGS]` (through the running instance, or a headless one started for the run), `script list`, `script daemon [--vault]` and `script install-client [DIR]`; `--control-stdio` takes `--no-scripts` and `--daemon`; `studyfrog --print-paths` lists `scripts`. Cron parsing is in-house (`studyfrog-service/src/scripting/`).
- Python API (STORY-0131): `python/studyfrog/`, a standard-library-only client of the control protocol with type hints and a README; `connect()` attaches to a running instance's socket (or `$STUDYFROG_SOCKET`) or starts a headless one, and offers `state`, `query`, `search`, `nav`, item and stack operations, rehearsal steps, `export`, `import_file`, `logs`, `subscribe`/`events` and `call` for any command. Its unittest suite runs against the real binary from `cargo test -p studyfrog-app --test python_client`.
- Lifecycle events (STORY-0130): every window and headless instance publishes `vault_opened`, `vault_closed`, `item_created`, `item_saved` and `rehearsal_finished` (derived from what it applies); the control command `subscribe [kinds...]` streams them as `{"event": "lifecycle", "data": {kind, at_ms, data}}` lines until the connection closes, and `studyfrog ctl subscribe` prints them. ADR 0013 decides how scripting works (out of process, over the control protocol).
- Zen mode (STORY-0116): Ctrl+Shift+F, F11 or the four-corners button in the top bar hides the sidebar and the status bar; Settings > Appearance > "Zen mode during rehearsal and exam runs" (`zen_in_runs`) turns it on by itself during runs. Primary buttons such as Show answer and Submit are at least 36 pt tall, and showing the answer never moves the prompt or the run's header.
- Calmer surfaces (STORY-0115): cards are set off by tone instead of 1 px outlines (a card in a card is an inset), buttons and fields are drawn by their fill (outlines on hover and focus, and all outlines back with high contrast), spacing follows 4 / 8 / 16 / 24 tokens that scale with density, and each view has one accent-filled primary action (the top bar's + Create and Home's diary Write entry are now quiet buttons with an accent label). Spec `docs/specs/ui-colour-and-type.md`.
- Typography (STORY-0114): a four-tier type scale (heading 1.40, body 1.00, label 0.92, caption 0.80 of the body size) used by every view and following the text size; prose has a 1.5 line height with an even first line and keeps to a reading measure of about 70 characters (help pages, item and note previews, rehearsal cards); timers and counters reserve their widest width so ticking digits never move their neighbours; Library's numeric columns are right-aligned. The rehearsal card is narrower and its answer is body text in the text colour. Spec `docs/specs/ui-colour-and-type.md`.
- Perceptual colour (STORY-0113): the palettes are OKLCH tokens (`theme_tokens.rs`) computed by in-house Oklab/OKLCH and APCA maths (`color_science.rs`); the six accents and the status colours of a theme share one perceptual lightness and chroma, surfaces climb in equal lightness steps, note text colours and item tints are chosen in OKLCH/Oklab, and a test checks the APCA contrast of every text/background pair in all 24 palettes (body text |Lc| >= 75, secondary >= 58, dividers 15 to 28; targets in `docs/specs/ui-colour-and-type.md`). The light theme's page is now a light grey with white cards; secondary text and borders in the dark theme are a little stronger.
- Image occlusion cards (STORY-0109): masks over a picture kept as an `occlusion` front-matter block (normalized `x y w h label`, `hide_all` or `hide_one`; the picture is never changed). "Make this an image occlusion card" on a card that shows a picture, or "Occlusion card from <picture>" for a picture or snapshot saved in the PDF viewer; draw, move, resize, label and delete masks on the card's page. A run asks one mask at a time (they take turns with the card's reviews), hidden and marked before the answer and uncovered after it; typed answers are compared with the mask's label. Spec `docs/specs/image-occlusion.md`.
- Contrastive drill (STORY-0111): "Compare A and B side by side" on a confused pair in Reports > Leeches, or "Contrastive drill" for 2 to 4 selected Library rows, opens a window with the prompts side by side and the shuffled answers to match, then the items side by side with the words only each answer has; the distinction is kept as a `> Distinction (vs [[...]])` hint on each item (typed answers ignore it) or as a comparison note linking all of them, updated on a later save and undoable. Spec `docs/specs/contrastive-drill.md`; `studyfrog --screenshot ... --screen contrast:<ULID>,<ULID>|compare:<ULID>,<ULID>`.
- Focus timer (STORY-0112): a stopwatch in the top bar starts 25/5, 50/10, 90/15 or custom work and break blocks (long break after every fourth) that keep running on every screen, with pause, resume, skip break, reset and a stack to count for; a finished block is recorded (store migration 26, `focus_block`) against the calendar study session it overlapped and a card offers "Start break" and "Log in Diary…" (a diary entry pre-filled with the minutes and stack). Spec `docs/specs/focus-timer.md`; `studyfrog --screenshot ... --screen focus|focus-done`.
- Cloze deletions (STORY-0108): `{{c1::answer}}` and `{{c1::answer::hint}}` in a card's front or a note's body; a run hides one group at a time as `[...]` or `[hint]`, the groups take turns from review to review (`cloze_group: N` fixes one), the answer shows the words in bold green and typed answers are compared with them; the Markdown toolbar's `[...]` button and Ctrl+Shift+C (Alt: same number) make a gap; the editor's preview shows the gaps; a cloze card needs no back. Spec `docs/specs/cloze.md`; `studyfrog --screenshot ... --screen run-answer` pictures a run with its first answer shown.
- Inline cards in notes (STORY-0110): a note's page offers "Extract / Sync N card(s)…" when its body holds `Term :: Definition` lines or `Q:`/`A:` and `??:`/`A:` pairs; the panel previews new, updated and unchanged items, "Sync cards" creates or updates cards and questions in the note's stack (each with `extracted_from`, `extract_key` and a `Source: [[Note#Heading]]` link back, schedules kept, no duplicates on a second sync), and "Undo the sync" takes it back. Spec `docs/specs/inline-extraction.md`; `studyfrog --screenshot ... --screen extract:<ULID>`.
- MCP (STORY-0118): `studyfrog://reports/leeches` and `analytics_get_report` `kind: leeches` (leeches and confusion pairs); `forecast_simulate_workload` gives simulated reviews and `study_minutes` per day for up to 365 days with stages, bands, exam pacers and true retention; the retention report carries true retention and the forgetting curves by stability tier.
- Workload against desired retention (STORY-0106): Reports > Workload and each stack's Reports tab simulate the cards at every target from 0.75 to 0.97 (reviews and minutes a day, cards remembered, cards per minute), mark the least-work target and recommend the one that keeps the most per minute; "Apply recommended retention" writes `request_retention` into `studyfrog.toml` (`[srs]`) or the stack's `_stack.toml`, keeping the rest of the file.
- Reports > Workload (STORY-0105): the reviews or study minutes ahead for 30, 90 or 365 days from 16 simulated futures of every card's FSRS or SM-2 state (your rating mix, missed days, new cards a day, the time reviews take in each stack), by stage (new, learning, young, mature) with a typical value and a low-to-busy range; the exam pacer of every stack with an exam or due date ahead (new cards a day, reviews and minutes a day, consolidation, share ready on the day), also on the stack's Summary and Reports tabs.
- Reports > Leeches and the stack Reports tab (STORY-0104): items that keep failing (repeated lapses, a recent streak of Hard/Again, far more time than the stack's other items, many reviews and still a short stability) with the reasons, and pairs of items forgotten close together that look alike, share a tag or link to each other; actions "Edit or split", "Pause", "Drill", and a contrastive drill of a pair (a practice run of exactly those items).
- Reports > Habits (STORY-0102, STORY-0103): recall, volume and median/p90 answer time by local hour and weekday (each review at the UTC offset of its own time), the 2-hour peak and low point with a choice of how many reviews an hour needs (5/10/20/50); recall and pace by minutes into a run and by position, the point where fatigue sets in and a recommended run length.
- During a run, a quiet line when an answer took far longer than the run's usual, with "Don't show this" and Settings > Rehearsal > Slow answers (`slow_answer_notice = false` in `app.toml`) (STORY-0103).
- True retention and forgetting curves by stability (STORY-0107): Reports > Memory and forecast shows true retention (mature items reviewed within 20 % of their interval), young, mature, early, on-time and overdue retention next to what FSRS expected, the memory half-life of a typical item (the model's and your own, fitted), and the measured vs predicted recall of each stability tier (<7, 7-30, 30-90, 90+ days) by `t/S`. `studyfrog --screenshot ... --screen reports:<tab>` opens a tab of the reports.
- Exam mode (EPIC-033): **Exam** next to Flip cards and Type answers in the rehearsal setup (STORY-0128). Every card and question once, answered in writing; no skipping, no going back, no answer shown, no rating; a submitted answer is locked and the next item follows (engine rules with property tests, STORY-0126). Each answer is compared with the back or the reference, `accepted` answers and `rubric` (STORY-0127); the result screen shows the percentage, the grade (A+ / 1+ from 95 % down to D / 4.0 at 60 %, F / 5.0 below), passed or not and a review of every answer. Exams never change the schedule; a stopped exam is not graded. Spec: `docs/specs/exam-mode.md` (STORY-0125); help page "Exam mode" (STORY-0129).
- Exam results in Reports > Runs (with an Exam filter) and as the diary prompt's note; `studyfrog --screenshot ... --screen exam-setup|exam|exam-result` (STORY-0128).
- Exam runs over the control protocol: `rehearse setup --mode exam`, `rehearse type`, `rehearse submit`, `rehearse finish`, refusals as errors, and `rehearse.run.exam` in `state` (STORY-0129, STORY-0124).
- Store migration 25: the `run` table accepts the mode `exam`; new `exam_result` table (STORY-0126).
- The assistant can use tools of external MCP servers (STORY-0121): register a program or an `http(s)://` address under Connections > Tool servers (MCP) (kept in `mcp_servers.toml` in the config folder), discover its tools, resources and prompts, and let the assistant call them while working the vault or examining you; only trusted read-only tools run without asking, every other call waits for "Allow this call", and a remote server gets nothing until you allow it.
- MCP authoring with the user in the loop (STORY-0119): the standard prompts are MCP prompts (`file_path`, `target_stack`, `depth`, `count`); `vault_propose_items`, `note_extract_inline_cards`, `diary_log_entry` and `calendar_plan_session` stage their changes in `.studyfrog/staged/` unless the server runs with `--mcp-allow-write`, and the Assistant's "Work the vault" tab lists them to review, apply or discard; `vault_move_stack` and `vault_delete_item` need `--mcp-allow-write --mcp-allow-destructive` and keep undo entries the user takes back under "Changes by AI tools".
- More of the MCP server (STORY-0118): resources `studyfrog://reports/{kind}` (retention, forecast, familiarity, calibration, runs, latency) and `studyfrog://diary/{date}`; an item's resource also carries its FSRS or SM-2 schedule in every stack and finds items by key; stacks by folder or title; the summary has due and new counts, items by kind and the streak; read-only tools `vault_get_due_queue`, `analytics_get_report`, `forecast_simulate_workload` and `pdf_read_pages`. Leech diagnostics and a study-minute forecast are not built yet.
- The MCP server can also listen on local HTTP: `studyfrog --mcp --mcp-http 127.0.0.1:<port>` serves `POST /mcp` (Streamable HTTP, JSON answers) on 127.0.0.1 only, wants a bearer token (`mcp_http` under `[keys]` in `secrets.toml`, else a fresh one printed to standard error) and refuses foreign `Host` and browser `Origin` headers against DNS rebinding (STORY-0120).
- A control protocol for AI agents and scripts: `studyfrog --control-stdio [--vault PATH]` runs headless and answers one JSON line per request (words such as `nav library`, `query type = card`, `item create card "Title" --stack french`, `rehearse flip`, or JSON with an id); the window and headless instances serve `run/control.sock` in the application home (with `run/control.json`), and `studyfrog ctl <command>` sends one request to the running instance; `state` returns a snapshot of the screen, vault, item page, dialog, rehearsal and notices, `screenshot` pictures the window, and `logs --follow` streams new `console.jsonl` entries; destructive commands need `--control-allow-destructive`, `--control-read-only` refuses writes, `--no-control` keeps the window from serving the socket (STORY-0124).
- Item keys: every new stack, card, note and question gets a short key such as `STACK_1`, `FLASHCARD_12`, `NOTE_3` or `QUESTION_7` in its front matter; "Assign keys…" in the Library gives older items theirs after a confirmation (undoable); the query language has `key`, an exact key typed into the top-bar or Library search finds its item, and keys show in a Library column, the item page's toolbar (with "Copy key") and the Details tab (STORY-0096).
- Item colours: a `color` in front matter (palette word or `"#RRGGBB"`), chosen in the item page's Details > Properties (swatches, None, picker, code); it tints the item page, its Details cards, the rehearsal card and the item's Library row (with a dot), keeping text at a 4.5:1 contrast in both themes (STORY-0097, STORY-0123).
- `studyfrog --screenshot FILE --screen item:<ULID>` pictures an item's page (STORY-0123).
- Store migration 24: `item.key`, `item.color` and a `key_counter` table, so a deleted item's key is not handed out again (STORY-0096).

### Changed
- The prompt search also matches the category name (STORY-0092); in Settings > Data the Copy buttons come before the paths (STORY-0122); "Test connection" sits on its own line under each connection (STORY-0083).
- PDF viewer pictures are kept by last use (true LRU): a page or thumbnail drawn again moves to the newest end of its cache, so the least recently used goes first (STORY-0085).
- The sidebar scrolls when the window is too low for all its screens (STORY-0133).
- On a narrow window the vault name in the top bar is cut short instead of being drawn under the search field, and the search field may shrink further, to make room for the focus timer (STORY-0112).
- `studyfrog-net` returns the answer's headers with every response, so an MCP client can keep a server's session (STORY-0121).

### Fixed
- The PDF page is never drawn larger than the graphics card's texture side or the renderer's 36-megapixel budget; 400 % on a 2x display drew nothing ("too big") and is now drawn at that limit (BUG-0010, STORY-0085).
- OKLCH gamut mapping stepped down before bisecting, since the gamut's slice is not convex near its corners (a pure blue did not round-trip) (STORY-0113).

## [0.5.0] - 2026-11-08

### Added
- A Model Context Protocol server for AI clients: a pure JSON-RPC 2.0 router with plain-data tools, resources and prompts and a policy for writes (STORY-0117), the read-only vault summary, stack and item resources and a search tool using the query language (STORY-0118), and `studyfrog --mcp [--vault PATH]` over standard input and output with "Copy MCP server config" in the Assistant's connections (STORY-0120).
- The Reports screen has "Export as CSV": one RFC 4180 table with the review total, retention, answer times, the review forecast, the familiarity distribution and the run log, written to `exports/` in the vault (STORY-0098).
- Many more built-in achievement tiers: streaks of 3, 14, 60, 100 and 365 days, runs up to 100, ratings up to 5,000, mastery up to 500, and four tiers of learned and mastered items for every stack; the ids of the first set are unchanged, so earlier unlocks stay (STORY-0086).
- The achievements screen has a filter (all, global, one stack), groups its entries by category and stack, and a stack's page shows how many of its achievements are unlocked (STORY-0087).
- Google Gemini (`generateContent`, key in the `x-goog-api-key` header only, safety stops reported with their reason) and DeepSeek (`/chat/completions`, default address and models) as providers of the assistant; choosing a protocol fills in its first suggested model (STORY-0083, STORY-0084).
- Four more standard prompts: explain a file at three levels, be examined on a file, learn a file by Socratic dialogue, and map the concepts of a file; their worked examples are read by the importer in the tests (STORY-0088, STORY-0089, STORY-0090, STORY-0091).
- Prompts have categories (create content, tutoring, analysis), a search, and options that are put into the text, such as the depth of an explanation or the number of exam questions; Save and the assistant run use the chosen options (STORY-0092).
- The prompt library is a screen of its own, "Prompts" in the sidebar; the Assistant's Prompts tab links to it (TASK-0233).
- StudyFrog keeps its own files in one application home (`$STUDYFROG_HOME`, else `$XDG_DATA_HOME/studyfrog`, else `~/.local/share/studyfrog`): `config/`, `models/`, `logs/`. Files of the old `~/.config/studyfrog` are copied in on first start and left in place; API keys end up readable by the owner only (STORY-0122).
- Every notice is written to a rotating text log (`logs/studyfrog.log`) and a JSON Lines log (`logs/console.jsonl`), 5 MB x 3 files each (STORY-0122).
- `studyfrog --print-paths` lists the application's directories, and `studyfrog logs [--tail N] [--follow] [--json]` shows its log (STORY-0122).
- Settings > Data lists the application files with a Copy button for each path (STORY-0122).
- Benchmarks for parsing a document and for drawing a thumbnail and a page at 2x, next to the existing page benchmark (STORY-0085).

### Changed
- The assistant's call log moved from `~/.local/state/studyfrog/agent.log` to `logs/agent.log` in the application home (STORY-0122).
- The PDF viewer trims full-size page pictures (6) and thumbnails (60) apart, so scrolling the sidebar thumbnails no longer pushes out the page being read and far fewer large page textures are kept in memory; the sidebar and the arrange grid share one thumbnail scale (STORY-0085).

### Fixed
- PDF page, sidebar thumbnail and arrange-grid textures are keyed by their pixel size instead of the requested scale, so a low-resolution thumbnail shown while the sharp render is in flight no longer takes over the sharp texture and leaves the page blurred at some zoom levels or fit modes (BUG-0010).

## [0.4.3] - 2026-11-06

### Fixed
- Both CSV importers (`studyfrog-ext-example` and `studyfrog-service::xfer_csv_map`) now merge unquoted multiline continuation rows in headed CSV files before splitting columns, fixing import failures such as `line 5: a card needs at least a front and a back` when AI-generated CSV exports contain unquoted newline breaks or numbered lists inside `front` or `back` fields (BUG-0014).

### Added
- Stacks can now be moved between a parent stack and standalone (top-level) after creation via `"Move stack…"` in the stack page's `"More actions"` menu and the `"Parent stack"` selector in the stack's `Properties` tab, while preserving all items' primary stack associations and SRS schedules (STORY-0100).
- Items (notes, cards, and questions) can now be deleted directly from the item editor toolbar (`"More actions"` -> `"Delete item…"`) with an inline confirmation prompt and full `"Undo last change"` support in the library bar (STORY-0101).

## [0.4.2] - 2026-11-05

### Fixed
- Rehearsal card prompts and answers now wrap cleanly across spans (`TextWrapMode::Wrap`) and scroll vertically when taller than the viewport, and CSV rows containing unquoted commas inside `front` or `back` fields (such as LLM-generated 6-column `stack,kind,title,front,back,tags` outputs) automatically reconstruct the full question, answer, and tags instead of truncating at the first comma (BUG-0012).
- The rehearsal result (`Run finished`) screen is wrapped in an outer vertical `ScrollArea`, the `Review` section stretches dynamically with viewport height inside its own inner `ScrollArea` (instead of being capped at 140 px), and every review item card always displays `Suggested rating:` alongside reading word count and time even when no rating override is needed (BUG-0013).

## [0.4.1] - 2026-11-05

### Fixed
- The card file importer automatically detects and parses both CSV (comma-separated, RFC 4180 quoted values, and multi-column AI prompt headers `stack,kind,title,front,back,tags`) and TSV (tab-separated) formats without requiring manual conversion (BUG-0011).

### Changed
- The item page details column is restructured into a four-tab notebook layout (`Properties`, `Schedule`, `Relations`, `Learning`) with inline mutable fields and direct stack due date editing (STORY-0093).
- The Create dialog exposes priority for stacks and due date / reminder selection across all creatable item types, syncing deadlines directly to the vault calendar (STORY-0094).
- Selecting Note in the Create dialog automatically expands the modal to fill the window and defaults the Markdown editor to Write mode while retaining drag-resize, shrink toggle, and item kind switching (STORY-0095).
- The top-bar Vaults switcher popup displays clean folder basenames in borderless selectable rows with automatic ancestor disambiguation for duplicate folder names and full paths on hover (STORY-0099).

## [0.4.0] - 2026-11-04

### Fixed
- Reading two Anki packages at the same time no longer shares one temporary file, which made the Anki import tests fail now and then.
- The Board's columns share the width there is, and the new-card field leaves room for its button, so the last column and "Add card" no longer run off the right edge (TASK-0226).
- PDFs are named by their file name in the Home card, the stack page, the search results and the viewer's header; the title inside the file is added only when it says something, so files whose title is a placeholder such as "PowerPoint-Präsentation" are told apart (BUG-0009).
- Enter in the Markdown editor continues a list: the next number, the same bullet or an unchecked box for a task item, at the same indent (in the middle of an item it splits the item); Enter or Backspace on an empty item removes its marker and ends the list (BUG-0007).
- Bullets, numbers and check boxes in the preview are level with the first line of their item (BUG-0008).
- The window buttons are anchored to the window's right edge, so the red close button is no longer cut off, and there is a gap between maximise and close (BUG-0005).
- The roadmap takes the width and the height there are, even when its dates are close together, with larger rows, bars and text (BUG-0006).

### Changed
- A rating in a run writes one review and one new memory state for every selected stack that holds the item, each with that stack's own scheduler and retention; queues treat an item held by several selected stacks as one candidate, due when it is due in any and new when any of them has not scheduled it; the daily limits and Home's due counts count each stack on its own; the calendar shows and moves an item's due date per stack (TASK-0218).
- Familiarity is worked out for each (item, stack) from the reviews that stack made and its own exam, and an item's overall value is the average over its stacks: the item page lists the average familiarity and difficulty and, for each stack, whether it is the item's own or a reference, its familiarity, difficulty and due date; the library column shows the average with the per-stack values on hover; the Needs attention card, the reports, the stack percentages, the queues and the familiarity replay use the stack's own value (TASK-0219).
- Reviews and the FSRS and SM-2 states are kept for each (item, stack) (schema v21): every existing row becomes the row of the item's primary stack, or of the Inbox for a loose item, and the store reads and writes a stack's own state (`*_in` functions). The screens still show the primary stack's value until the per-stack views arrive; imported review histories are written after the reindex so they land in the item's stack (TASK-0218).
- Decided and written down: stack folders, items referenced by several stacks and an independent memory per stack ([ADR 0011](docs/adr/0011-stack-folders-and-references.md), `markdown-schema.md`, `srs-algorithm.md`; TASK-0210). The implementation follows in TASK-0211 and the tasks after it.
- The Markdown editor's code is split into a module of small files (`view/md_editor/`: toolbar, resizing, preview cache, list keys).

### Added
- Documentation, help and release notes for PDF export: in-app help page and FAQ entry explaining export routes, options, supported and unsupported elements; comprehensive showcase note demonstrating all supported elements in `examples/pdf-export/complete-note.md`; round-trip export and text extraction test in `studyfrog-service`; manual window verification and draft release notes for 0.4.0 (TASK-0232).
- Export to PDF in the application: dialog with page size, margins, font size, title page, TOC, page numbers, and card/question answers; live preview of page count and sample rendered pages before writing; entry points in item page More actions, library bulk bar, stack page More actions, and the Transfer/Export screen; notifications and toasts that open the exported PDF directly in the viewer (TASK-0231).
- Self-assessment against measured results: pure calibration report in `studyfrog-report` correlating self-rated ratings (understanding, retention, focus, mood) against objective review accuracy and item familiarity; bias analysis identifying overconfidence or underconfidence; dedicated "Self-assessment" tab in the Reports screen with metric cards, entry count indicators, and chronological calibration timeline (TASK-0174).
- Prompt for a diary entry after a rehearsal: compact self-assessment card on the rehearsal summary screen pre-filling session duration, linked run ULID, stack name, and reviewed item count; exposes 1..=5 buttons for user primary scales (energy, focus, satisfaction) with descriptive level tooltips; skippable with one key (Esc) without delaying leaving the summary (TASK-0173).
- Export use cases for PDF in the service: pure export planning and transactional execution in `studyfrog-service` for single items, selected item sets, and stacks (including sub-stacks and referenced items deduplicated exactly once); resolves internal wikilinks and vault picture attachments with labelled placeholder fallback; atomic writes via temp file and rename; hostile path refusal; cooperative cancellation and progress reporting (TASK-0230).
- Diary screen: full-featured Diary screen supporting list and month-calendar views, rich draft entry creation and editing form, 13 standard self-assessment scales with level descriptions, user-defined custom scales in Settings > Diary and reflections, tag, stack, and rating filters, date navigation, and delete confirmations (TASK-0172).
- Markdown layout and PDF writer: pure layout engine in `studyfrog-pdf` paginating Markdown AST into pages with wrapped text, syntax-highlighted code blocks, tables with aligned columns, list items, task checkboxes, and blockquotes; PDF writer built on `lopdf` producing searchable PDFs with TrueType `/ToUnicode` CMaps, document bookmarks (`/Outlines`), clickable web and internal wikilinks, and document metadata (TASK-0229).
- Folders view and Items view in the application: sidebar navigation distinguishes Folders (Obsidian-modelled file tree explorer) and Items (catalog table); Folders view includes a top toolbar (New note, New folder with stack option, Sort order, Expand/collapse all), virtualized indent tree rendering, file kind badges, context menus, drag-and-drop moves, cross-stack move confirmation dialogs, inline rename (`F2`), and keyboard navigation (TASK-0204).
- Diary entries in data model, Markdown, and index: `DiaryEntry` and rating types in `studyfrog-core`; `kind: diary` frontmatter parsing, rendering, and lossless round-tripping in `studyfrog-md`; schema migration 23 and index tables (`diary_entries`, `diary_runs`, `diary_stacks`, `diary_items`, `diary_exams`, `diary_tags`) with day-listing, text search, and item-link queries in `studyfrog-store` (TASK-0171).
- Folder operations in the service: pure plans and atomic transactional execution for creating, renaming, moving, and deleting folders and files inside the vault; link and attachment rewriting for moved files; cross-stack boundary detection with primary stack transitions and `referenced_by` adjustment; rollback on failure with undo snapshot support; strict refusal of hostile names, escape sequences, and protected roots (TASK-0203).
- Specification of the diary: comprehensive specification in `docs/specs/diary.md` defining daily entry formats, 13 standard self-assessment scales with exact wording, user-defined scales in TOML, bidirectional run and stack links, and non-intrusive post-rehearsal prompts (TASK-0170).
- Specification of folders and items views: specification in `docs/specs/vault-views.md` defining the Obsidian-modelled Folders view and structured Items view with toolbar actions, stack vs normal folder distinctions, and keyboard navigation (TASK-0202).
- Specification and architecture decision for Markdown-to-PDF export: `docs/specs/pdf-export.md` and `docs/adr/0012-markdown-to-pdf-writer.md` selecting an in-house layout engine on `lopdf` under Rule P10 (TASK-0228).
- Export formats TOML and RON next to JSON: export the whole library or a stack to JSON, TOML or RON with identical data (items, references, and in advanced mode per-stack scheduling values: source, difficulty, familiarity, due date, average, and review history); all three formats round-trip back through import and are parsed and validated with standard ecosystem crates; the export dialog displays re-importability notes (TASK-0220).
- Mark a folder as a stack or as a normal folder: a normal folder can be marked as a stack (creates `_stack.md` with a new id and adopts items), and a stack can be turned into a normal folder via the stack page's More actions; unmarking opens a confirmation modal detailing all changes (settings, status, goals, subjects, sub-stacks, referencing items, and target stack/Inbox); the change can be undone via toast; the Create dialog provides a "Mark as stack folder" choice to create normal folders (TASK-0217).
- Stacks & references in the interface: the item page shows the primary stack and the stacks that reference the item, with Add to stack and Remove from stack buttons and the schedule, due date and familiarity of each; items can be added to another stack from the library bulk bar or by dragging onto a stack; stack pages (library table, backlog, board, roadmap) mark only-referenced items with `(from {primary})`; the Create dialog keeps the parent stack as primary and offers an optional "Also in" picker; every control has an accessible name and keyboard navigation (TASK-0213).
- An item referenced by a second stack counts in that stack's summary, sub-stack tree, reports, familiarity report, assistant context and runs (a run over either stack includes it, and a run over both queues it once); the library has an Also in column; JSON export and import, CSV (an `also_in` column when needed) and imports carry the references and create a stack they name (TASK-0212).
- An item can say which other stacks reference it (`referenced_by`, a list of stack ids in its front matter); the index has a table and a view of which items belong to which stack, and the vault check reports a reference to a stack that does not exist, to the item's own stack or listed twice. Vaults without the key read as before (TASK-0211).
- Run a standard prompt through the connected assistant: choose a prompt and a text file, read exactly what would be sent, press Send, read the answer and import its CSV through the preview; if the preview finds problems, "Ask the assistant to fix these" prepares a message with those rows, sent only after Send (TASK-0207).
- Code blocks with a language (python, r, rust, php, java, cpp, c, toml, json, html, xml, csv, tsv) are coloured: keywords, strings, comments and numbers, in colours that read well in both themes; other languages are drawn plain (TASK-0214).
- Text macros: `{table}`, `{table:3x4}`, `{toc}`, `{code}`, `{code:python}`, `{quote}`, `{image}` and `{color:red}` turn into Markdown when the closing brace is typed; the `{ }` button in the toolbar lists them (TASK-0198).
- Tables (pipe tables with column alignment), underline, text colours from a small palette that works in both themes, strikethrough and an automatic table of contents (a line `[TOC]`) in the Markdown preview, the item page, the cards of a run and help; the toolbar has Table, Underline, Strikethrough and Colour. Underline and colour are kept as the small HTML other readers understand; any other HTML stays visible as typed (TASK-0197, TASK-0215, TASK-0216).
- Six standard prompts for any AI chat (flashcards, questions and answers, a whole stack, notes, summary, critique) that tell the AI exactly which CSV to write, for attached text, PDF text, source code and data files; the Assistant screen has a Prompts tab to copy or save them (one or all), and the import dialog offers a prompt next to the CSV format. Their worked examples are tested against the importer (TASK-0205, TASK-0206).
- CSV and TSV import with column mapping: the delimiter, the encoding (UTF-8 or Latin-1), a header or none and which column is the front, back, title, tags, stack and kind are detected and can be changed under the preview, which shows the first rows; a two-column file is front and back; a kind column can make notes and questions; every problem is listed with its row and column, and "Copy the problems for the AI" puts a ready message on the clipboard (TASK-0208).
- The import dialog lists the formats it reads with the layout each expects, an example and a Save example button; an unreadable file's error names the expected layout; the help has an Import formats page generated from the same table (TASK-0209).
- The editor saves by itself when a word is finished (a space or line break is typed), when something other than text changes, and at the latest 5 seconds after the first unsaved edit; a pause alone saves nothing. Leaving, opening another item and exiting save first; a file changed outside StudyFrog is never overwritten (Reload from disk or Keep mine); a failed save is shown and retried (TASK-0200).
- Leaving an item you changed shows a "Saved <title>" toast that goes away after about three seconds (TASK-0201).
- The notice after Create has an Open button that opens the new item, and the same entry in the notification list keeps that link (TASK-0199).
- `studyfrog --screenshot FILE --theme dark|light|system` takes the picture in a theme, whatever is saved (TASK-0226).
- Colours in the views come from the theme: the calendar's appointments and the graph's notes use a new theme colour, the notification badge the palette's red, and a test fails when a view writes a colour of its own (TASK-0226).
- Appearance settings (Settings > Appearance, the theme also in the profile menu): dark, light or follow the system; the interface size (70 to 200 percent, also with Ctrl and plus, minus and 0) and the text size; compact, comfortable or spacious density; six accent colours; high contrast; reduced motion; the screen to open first (Home, Library or the one used last); the sidebar folded at start; and the window's size and place remembered. Every change shows at once and is saved at once, with no restart, and the saved look and window are used from the first frame. The settings are in an `[appearance]` table of `app.toml` (`docs/specs/appearance.md`) (TASK-0221 to TASK-0225, TASK-0227).
- Navigation history: the top bar has Back and Forward arrows and a History list (the screens, stacks and items visited, newest first, the current one marked), and Alt+Left and Alt+Right do the same; unsaved changes in the editor ask you to save or revert first (TASK-0196).

## [0.3.1] - 2026-11-03

### Fixed
- Scroll bars reserve their own width, so widgets and tables next to them no longer run under them (TASK-0193).
- Rows of a name and a bar (priorities, ratings, familiarity spread) are aligned: the names share a column and every bar starts at the same place and has the same length, with its number on it; the numbers of Types and the due rows on Home line up as well (TASK-0192).
- Every screen has a wider margin (20 px at the sides, 14 px at the top) and the stack page no longer sits against the stack list (BUG-0002).
- The Create dialog's header is two rows, the window controls on one and the stack and type on the other, so the type name is no longer cut off in the normal-width dialog (BUG-0003).
- The four figures at the top of Home (items ready, streak, reviews, items) are spaced further apart (BUG-0004).

### Changed
- StudyFrog draws its own title bar: the top bar moves the window when dragged (the window manager does the moving, so snapping works) and maximises it on a double click, with large minimise, maximise and a red close button (which replaces the separate Exit button and asks about unsaved changes); the edges resize the window (TASK-0190).
- A calmer, easier to read layout: a reading column for the rehearsal setup, Settings and help; grouped cards and captions instead of large headings; the rehearsal setup keeps its Back and Next buttons in view; the library's search is one row and its stack list has a fixed width; the stack page has one primary button and a "More actions" menu, and a two-column Summary (TASK-0192, `docs/specs/ui-calm.md`).
- The status bar no longer has a Help button; the `?` in the top bar, Help in the sidebar and `F1` remain (TASK-0192).

### Added
- The rehearsal screen has a full-width progress bar with "N done · M left" counted against the items still open: an item with its answer showing is still open until it is rated or skipped, and items put back to repeat count as open and are named (TASK-0194).
- `studyfrog --screenshot FILE [--screen NAME] [--size WxH]` saves a PNG of a screen of the running app and exits, for looking at the real rendering (`home`, `library`, `rehearse`, `run`, `settings`, `help`, `calendar`, `board`, `reports`, `agent`, or `stack:<folder>`) (TASK-0192).
- Cancel, the close button, Escape and a click outside the Create dialog ask "Discard what you wrote?" when a field has content or a changed value; an untouched dialog closes at once (TASK-0188).
- A red button at the right end of the top bar ("Exit StudyFrog") closes StudyFrog and asks first when the editor holds unsaved changes (TASK-0189).
- The Import tab lists what it finds on the computer: Obsidian vaults (from Obsidian's own list, with the number of Markdown files), Anki packages in Downloads, Documents and Desktop, and Anki profiles with the hint to export a package; "Use" shows the preview for the chosen one and nothing is read before (TASK-0191).
- A proposal for a calmer UI in `docs/specs/ui-calm.md` for the owner to accept (TASK-0192).

## [0.3.0] - 2026-11-03

### Fixed
- Teacher and subject names may contain commas ("Prof. Dr. Künkelmann, Harry"): a stack's `subjects` list is written as a
  quoted list when needed and read back whole, and the Settings screen refuses a name that cannot be stored before saving,
  keeps what was typed and says why next to the field (BUG-0001).

### Changed
- The `?` in the top bar opens the help page of the screen instead of a menu of shortcuts (they are on the Keyboard shortcuts page) (TASK-0162).
- The Create dialog is laid out like Jira's: the parent stack and the type (a drop-down) in the top left corner, a large title, the Markdown editors, then priority, difficulty and tags (due date for a stack); "Add PDF ..." moved into a More menu, and Enlarge and a grip at the lower left widen the dialog (TASK-0186).
- Every Markdown editor, in the dialog and on the item page, gets a bar under it that makes it taller when dragged or with the arrow keys (TASK-0187).
- New cards, notes and questions start with priority medium and difficulty difficult (the highest) unless the user chose others; stacks get neither (TASK-0185).
- The library page no longer holds the new-card form: a card, note or question is made in the Create dialog, and the import panel and the card types of extensions moved into an "Import…" dialog on a stack's list (TASK-0134).
- The bell's notifications are records the user manages: each has an id, time and read state and is kept in the vault's database for 30 days
  (at most 200), so they survive a restart. Opening the panel no longer marks everything read (TASK-0126).
- The top bar shows the open vault's folder name next to StudyFrog (cut at 24 characters, full path in the tooltip), reversing the
  removal of STORY-0041 at the owner's request (TASK-0128).

### Added
- Help inside the app (`F1`, the `?` in the top bar, or Help in the sidebar, with or without a vault): a page tree (Start, Screens, Reference, Your pages, Tutorials), the page for the screen that is showing, links between pages, Back and "Back to" the screen, and a search over titles and text. The 26 supplied pages are compiled in from `docs/help/` and read-only; a test fails when one links to a page that does not exist (TASK-0161, TASK-0162).
- The help pages themselves: Why StudyFrog, Getting started, Privacy and your data, a how-to for every screen, the FAQ, the glossary, keyboard shortcuts, the query language, familiarity and statuses (TASK-0165).
- Own help pages: New page, Copy to my pages (from a supplied page), Edit, rename and Delete; each is a Markdown file in the vault's `wiki/` folder (not indexed as items, part of backups), searched and linked like the supplied pages (TASK-0163).
- The assistant can read the user's help pages ("Also read my help pages" on its wiki tab, shown in the preview before Send) and propose new pages or edits; nothing changes until the user applies them, and supplied pages are never sent nor offered for change. Help and the assistant link to each other (TASK-0164).
- A "First steps" tutorial: offered once after the first vault opens (dismiss to skip; `tutorial_offered` in `app.toml`), repeatable from Help; a card at the lower right walks through creating a stack and a card, rehearsing and planning a session, opening each screen and marking a step done when the user has done it (TASK-0166).
- An import and export dialog (from the Create dialog's More menu, a stack's page and Settings under Data): Export asks for the mode (Default without scheduling, or Advanced with the review history), the format (JSON zip, CSV or Markdown zip) and the folder; Import picks a file or folder, shows a preview with counts, warnings and each item's verdict, lets the user skip, replace or keep both for duplicates, and writes nothing on a cancel (TASK-0160).
- Importers for Anki (`.apkg`: decks to stacks, basic and cloze notes to cards, tags and pictures kept, the review log imported as reviews with the schedule rebuilt on request) and Obsidian (a vault folder or zip: folders to stacks, Markdown files to notes with front matter, wikilinks, tags and pictures; callouts, dataview blocks, comments, highlights, block references and note embeds stay as text and are counted for the preview), read through a table of format adapters that `docs/EXTENDING.md` shows how to extend (TASK-0167, TASK-0168, TASK-0169).
- Export and import engine (`docs/specs/interchange.md`): the library or one stack as JSON (versioned, with the review history only in the advanced mode) with pictures and PDFs in a zip, as CSV, or as the vault's Markdown files in a zip; import reads them into a bundle, a dry run reports what would be created, what exists and what fails, and duplicates are skipped, replaced or kept as a second copy. The existing card import runs through the same path. Hostile paths and files are refused (TASK-0157, TASK-0158, TASK-0159).
- The review after a run and a new Runs tab in the reports show each run's mode, pacing, planned time (countdown), time used and answers a minute; the Runs tab filters the history by mode (TASK-0182).
- Run setup offers a time for the run: no clock, a countdown (5, 10, 15 or 25 minutes or any from 1 to 180) or a count up. The rehearsal screen shows the time left or used, says "One minute left" and "Ten seconds left" in text and announces them to screen readers, and a countdown ends the run by itself at zero, leaving the item on screen unrated and opening the review (TASK-0181).
- The stack page has Board (a column per status, items move by dragging or a menu), Calendar (the month grid, day list and planned reviews of that stack only, sharing the global calendar's code) and Reports (the stack's familiarity and least familiar items) tabs; tabs that do not fit go into a More menu (TASK-0156).
- The stack page has a Roadmap tab: a row for each stack inside it and each item directly in it, with a bar from its first date to its next one on a time axis with a line for today; zoom (Today, Weeks, Months, Quarters), scrolling both ways, only the rows in view are drawn (1000 rows measured), and each bar is a named button that opens its stack or item (TASK-0155).
- The stack page has a Goals tab: a Markdown bullet list (check boxes allowed) edited in place and saved under a `## Goals` heading in the stack's `_stack.md`, where it is searchable and part of backups; the rest of the file is kept (TASK-0154).
- The stack page has a Backlog tab (Up next and Not yet planned with the count per status; an item moves between them by dragging or by a button, which makes it Learning or puts it on hold) and a Sub-stacks tab (the tree of stacks inside, folding, each with status, progress, due date and familiarity, opening one on a click) (TASK-0152, TASK-0153).
- A stack opens as a page like Jira's project view: a breadcrumb, the stack's title, a status chip that changes the stack's status, and the actions Details, New item, Import and Start rehearsal, with tabs Summary and List (the last tab is remembered per stack; Left and Right switch). The old stack page is the List tab (TASK-0150).
- The Summary tab, read only when it opens, shows cards for completed, updated, created and due-soon items, a status ring, priorities, types, an activity feed and the sub-stacks with status, progress and familiarity, all with their numbers as text (TASK-0151).
- Settings (Study tab) tells from the user's own review history whether familiarity improves the predictions of recall: a replay scores plain FSRS against the familiarity-adjusted prediction by log loss and a calibration table, fits the strength and half-stability on the first 80 percent when there are at least 1000 reviews, and shows the verdict, the best values and the change in reviews; it never switches the factor off itself (TASK-0184).
- A Familiarity tab in the reports shows how the items spread over the scale and, per stack and per subject, the roll-up and the three least familiar items with the inputs behind each value; the item page has a Familiarity card with the value, its drivers and its course over time (TASK-0138).
- Home shows a "Needs attention" card with the five least familiar items and why (low familiarity, overdue, an exam within 14 days); run setup offers "Least familiar first" and a familiarity filter (TASK-0139).
- Queues use familiarity: under a daily allowance the least familiar due items are chosen first, `familiarity_threshold` in `studyfrog.toml` leaves out due items at or above it, and a run can be ordered "Least familiar first" or limited to items less familiar than a level (TASK-0140).
- The calendar shows the reviews the schedule planned as chips ("French: 4 of 10 due") told apart from sessions: a click opens the stack, a chip can be dragged to another day, and the side panel and the context menu move it by date for those who do not drag; "Undo last move" takes a move back and a move past an exam or due date says so. "Show the reviews the schedule plans" turns them off (`calendar_reviews = false` in `app.toml`) (TASK-0148).
- Planned reviews can be read from the schedule and moved to another day (`review_plan` in the service): a move changes only the due date of the items, warns when the new day is after an exam or deadline of their stack, and can be undone (TASK-0147).
- The calendar crate has a planner (`review_plan`) that turns the schedule into entries, one per stack and day with the item count and the stack's size; done and paused items and stacks make none (TASK-0146).
- Library rows can be ticked (a check box per row and one for all rows the filter lets through); a bar then sets status, priority or difficulty, moves the items to a stack, adds or removes a tag, or deletes them after a question that names the number. Each change is one step that "Undo last change" takes back (TASK-0133).
- Saved filters: the Filters menu saves the current filter or query under a name in `filters.toml` in the vault, reopens it and removes it (TASK-0133; per vault, not per profile, as the library spec proposes).
- The library's search bar has a Standard mode (title word, type, status and priority) and a Query mode with the errors under the box, suggestions for field names, operators and values, and a switch that turns the fields into a query and a plain query back into fields (TASK-0132).
- The library list: every item of the vault in a table with sortable columns (title, type, status, priority, difficulty, familiarity, stack, subjects, tags, reviews,
  due, created, updated, path; a click sorts, a second click reverses, shift adds a key), a column menu that is remembered, 'N of M items', and a tree entry 'All items';
  10 000 rows draw in about 11 ms per frame because only the rows in view are built. The search is still the title search (TASK-0131).
- The library's query language: a parser with messages that name the column, a printer, and an evaluator over item rows (fields for text, type, status, stack, subject,
  teacher, tag, priority, difficulty, familiarity, due, reviews, dates and path; `AND`, `OR`, `NOT`, `IN`, `IS EMPTY`, `ORDER BY`). User text never reaches SQL; 10 000 items are
  filtered in a few milliseconds. The screens come with TASK-0131 and TASK-0132 (TASK-0130).
- Answer speed counts for familiarity and scheduling: each rating stores how long its answer was on screen and how long it should take to read (schema version 17),
  views store the reading time of the item, and a card read faster than expected gains more familiarity than one read slowly. Gains close to an exam or due date of the
  stack count for less (half on the day itself, in full from 14 days before), at the owner's request (TASK-0137, TASK-0141).
- The rehearsal engine knows two more pacings besides the free run: a countdown that ends the run by itself when its active time is up (the item on screen
  counts as not answered, with no rating and no schedule change; a pause stops the clock) and a count up. The pacing is stored with the run (schema
  version 16, older runs read as free) and a `time_up` event marks the end of a countdown. The screens come with TASK-0181 (TASK-0179, TASK-0180).
- Specifications of the library redesign (`docs/specs/library.md`: a Jira-style list with columns, sorting, selection and bulk actions) and of the
  query language (`docs/specs/query.md`: grammar, fields, operators, error messages, limits) (TASK-0129).
- A notification center under the bell: filters for all, unread and errors, mark one or all read or unread, delete one or clear all, the list
  grouped under Today, Yesterday and the date, a link on notifications that have a target (reminders open the calendar), and an unread count on
  the bell (TASK-0127).
- Familiarity values and their use in scheduling: `studyfrog_report::familiarity` values an item from its ratings, views, mark and the fall without
  contact; after each rating the next due date is multiplied by a factor that compares familiarity with what the FSRS stability suggests (0.25 to 2,
  nothing before the third review, never stretched past an exam or deadline, the stored state untouched). On by default; `familiarity_strength` in
  `studyfrog.toml` (0 is off) and `familiarity = false` in a `_stack.toml` switch it. Validation numbers are in `docs/specs/reports.md` (TASK-0137,
  TASK-0141).
- The item page times how long an item is looked at: active time only (a minute without input stops the clock), one `view` record per
  visit and an `edit` record when the item was changed, kept in the database and in backups. Settings, under Study and scheduling,
  can switch it off (`track_views = false` in `app.toml`) (TASK-0136).
- Statuses on screen: a status menu on the item page (Automatic, Not studied yet, Learning, In review, Complete, On hold), the status
  on every item in a stack's list, and a line under a stack's heading with how many of its items, sub-stacks included, are in each
  status (TASK-0145).
- Item status in the data: the optional `status` front-matter key (`new`, `learning`, `reviewing`, `done`, `paused`) on every kind, an
  index column (schema version 12) and an effective status on every item list, derived from the review history when the key is
  absent. The screens come with TASK-0145 (TASK-0144).
- Specification of the familiarity value (`docs/specs/familiarity.md`): update rules for ratings, views and view time, the slow fall
  and when it pauses, notes, the stack roll-up, confidence, worked examples and the properties to test (TASK-0135).
- Settings are in tabs by domain (Study and scheduling, Rehearsal, Answers and AI, Data, Subjects and teachers); the last tab is
  remembered in `app.toml` as `settings_tab`, and the Left and Right keys move between tabs. Tabs for General, Appearance and
  Notifications appear when they have settings (TASK-0142).
- Specification of item statuses (`docs/specs/status.md`): five statuses, where they are stored, who may change them, what they do to
  the schedule, and how a stack rolls up its items (TASK-0143).
- Priority order for 0.3.0 in four waves in `docs/roadmap/M10-usability-and-insight.md`; wave 1 is `ready` on the board (BUG-0001,
  TASK-0128, TASK-0135, TASK-0142, TASK-0143).
- Research and a proposal for combining familiarity with FSRS: a relative factor on the FSRS interval that leaves the stored
  state untouched, a reproducible simulation (`docs/research/familiarity-fsrs-sim.py`) and a replay that tells each user whether it
  helps; it is on from the start (TASK-0183, TASK-0184).
- Planning for 0.3.0 (phase M10, EPIC-023): notification management, the vault in the bar, a library with an SQL-like
  search, a stack overview with sub-views and a separate stack detail view, a familiarity value used in reports and scheduling, tabbed
  settings, item statuses, calendar entries for the next reviews that can be moved by drag and drop, import and export
  with Anki and Obsidian importers, a built-in wiki with FAQ and tutorials, a diary and rehearsal modes for countdown and count up, recorded as STORY-0055 to
  STORY-0070, BUG-0001 and TASK-0126 to TASK-0184; the issue checker accepts phase `m10`.
- Releases carry a `.tar.gz` (binary, desktop entry, icon, licences, README, changelog, release notes) and its `.sha256`
  next to the AppImage; the packaging script writes them and the release workflow uploads them (TASK-0125).

## [0.2.0] - 2026-10-31

### Added
- Links inside a PDF: outlined on the page, page links followed with a click, web and mail addresses opened only after
  the user presses Open on a question that shows the address; launch, script and other actions are never run (TASK-0123).
- Documentation, tests and benchmarks for PDF support: `docs/specs/pdf.md`, headless view tests with the accessibility
  audit, a 500-page benchmark with budgets, and draft release notes `docs/RELEASE_NOTES_0.2.0.md` (TASK-0122).
- PDFs in the library: the stack page lists its PDFs with page counts, 'Add PDF ...' (also in the Create dialog) copies a
  file into the stack under a safe name, renaming a PDF moves its notes file and rewrites every note, card and link that
  names it, `[text](file.pdf#page=3)` and `![](file.pdf#page=3)` open the viewer at that page, and the home page lists the
  PDFs that changed most recently (TASK-0121).
- Cards and questions from PDFs: a highlighted passage or the selected words become a card or a question with a link to
  the page, shown as a 'Source' link while rehearsing; the assistant can read chosen PDF pages (shown first, sent only
  after Send) and propose cards and questions that name the page they come from (TASK-0120).
- Pictures from PDFs: save the pictures of a page or a dragged rectangle of a page (figures, formulas) into the vault's
  assets folder under the usual picture rules, and put them at the end of a note made from the page (TASK-0119).
- Notes from PDF text: the text of pages or of a selection becomes a Markdown note in a stack, with a `source` key that
  leads back to the page; the item page offers 'Open source' and the viewer lists the notes made from the file
  (TASK-0117). Pages that are only pictures are flagged in the index and the viewer, and notes made from them say so
  (TASK-0118). `docs/specs/pdf.md` describes the formats and limits.
- Highlights and notes on PDFs: choose a colour, highlight the selected words or place a note, edit and delete them in a
  list, jump to each; stored in plain text in `name.pdf.notes.toml` next to the PDF, searchable, included in backups, and
  followed to the right page if the PDF is replaced by a revision (TASK-0114).
- Rearranging PDFs: turn, move and drop pages in a thumbnail grid with undo, append other PDFs, split by page ranges and
  save as a new file with new title, author and subject and, if wanted, the highlights and notes written into the copy
  as standard annotations; the original is never changed (TASK-0115, TASK-0116).
- PDF files in the vault are indexed (title, page count and the text of every page, extracted once per change) and the
  global search finds them by title and by text, showing the page of the hit (TASK-0111).
- The PDF viewer: one page at a time with zoom (fit width, fit page, percent), page jump, outline, page thumbnails,
  search inside the file with highlighted hits, word selection by dragging and copy, and the page's text for screen
  readers; pages are drawn on the worker thread and cached (TASK-0112, TASK-0113).
- The `studyfrog-pdf` crate (TASK-0110, TASK-0113): reads page sizes, outline, properties, words with boxes, text,
  Markdown and pictures; draws pages and regions; rotates, drops, reorders, merges and splits pages; writes
  highlights and notes into a copy. Size, page, pixel and time limits and crash containment keep hostile files from
  taking the app down.
- PDF library spike (TASK-0109): measured findings in `docs/research/pdf-library-spike.md` and a proposed ADR 0010
  (hayro to draw pages, pdf_oxide to read and edit, both behind a `studyfrog-pdf` crate); ADR 0010 is accepted.
- Planning for PDF support: EPIC-022 with STORY-0054 and TASK-0109 to TASK-0122 (view, annotate, rearrange and extract
  PDF files), a new phase M9 in the roadmap and the `m9` label.

## [0.1.0] - 2026-10-31

### Added
- The Assistant screen: connect a local model (Ollama and similar) or a remote one (OpenAI-compatible, Anthropic),
  let it read chosen stacks and propose changes like an LLM wiki editor (new notes, cards, questions, edits, an
  index) that you accept one by one, and be examined by it with the verdicts recorded as reviews only when you say
  so. A remote model is refused until you allow it; keys live in a private file in your config folder; calls are
  logged without their content (STORY-0049, ADR 0009).
- Backups and restores (Settings, and the welcome page): one plain `.tar` with the vault's files, a consistent copy of
  the database (review history, runs, the board and the calendar), and optionally the settings and API keys. A
  restore goes into a new or empty folder, never overwrites anything, and keeps the old settings next to the new
  ones (STORY-0050).
- User profiles: each has its own settings, recent vaults and API keys; switch, create and delete them from the
  profile menu (or the welcome page). The vault path is no longer shown in the profile menu (STORY-0051).
- Pictures in notes, cards, questions and answers: a Picture button in the Markdown toolbar copies a png, jpg or gif
  into the vault's `assets/` folder and inserts `![alt](assets/name.png)`; pictures show in the editor preview, the
  item page and while rehearsing, and only pictures inside the vault are loaded (STORY-0044).
- A Performance tab on the reports screen: reviews, items reviewed, good-or-easy share, answer times, rating bars
  and a 30-day chart for the whole vault, a stack, a subject or one item, over all time, 30 or 7 days, drilling
  down by clicking (STORY-0048).
- The item page: title, a Markdown editor for every text area (toolbar, Write, Split or Preview), and a side column
  with the kind, stack, subjects, priority, difficulty, tags, dates, the links to the item and the items that link
  to it (STORY-0042, STORY-0043). The Create dialog uses the same editor, wide for notes.
- Priority (lowest to highest) and difficulty for stacks, notes, cards and questions, inherited from the stack;
  priority orders spaced runs and breaks ties in the other orders, and the owner's difficulty stands in for the
  memory difficulty of new items (STORY-0053).
- Teachers and subjects (`subjects.toml`, edited in Settings); stacks name subjects and what is inside takes them
  (STORY-0045).
- A date picker (a month grid under the date field) in the Create dialog and the calendar form; a stack's due date
  can be set or changed from its Library page (STORY-0046, STORY-0052).
- Stacks whose due date has passed are left out of the suggested run and the home page's counts; choosing such a
  stack by hand in the run setup still works (STORY-0052).
- Cards made by an import or by the Library card form also get a board card (the first 25 of an import, then one
  summary card); board cards are draggable by their whole face and columns light up as drop targets (STORY-0047).
- ADR 0009 (accepted): how an AI agent will work the vault, where its keys and logs live, and that it only
  proposes changes (STORY-0049). New backlog stories for backups, profiles, priority and difficulty
  (STORY-0050, 0051, 0053).
- The end-of-run review is a summary with tiles and bars and a scrollable card for every item; prompts and answers
  are drawn as Markdown while rehearsing (STORY-0040).
- The vault's folder name and path no longer appear in the top bar, the home page, the status line or an "Opened"
  notice; the status line shows the size of the vault instead (STORY-0041).
- Calendar entries are now study sessions, tasks, appointments, exams or deadlines, each with up to five reminders
  that show once as a toast and a notification while the app is open; the month grid fills the window with
  coloured entries and a "Coming up" list (TASK-0105).
- A stack can be given a due date and reminders in the Create dialog, which puts a deadline on the calendar; every
  item made in the dialog also gets a card in the first board column (TASK-0106).
- The run setup is three steps: What, How, Review (TASK-0107). The home page lists the items due now and the next
  seven days and fills the window (TASK-0108).
- `docs/specs/UX_EVIDENCE.md`: the usability research behind these changes, with how strong each finding is
  (TASK-0104).
- A home page (greeting, items ready today, streak, stacks waiting with a Rehearse button, planned sessions,
  recent items and runs) that a vault opens on; a shared theme (palette, text sizes, spacing) for every screen;
  stacks as cards and items as rows in the library; a card layout, progress bar and coloured rating buttons in
  the rehearsal screen (TASK-0100 to 0103).
- A Create dialog (modal) for stacks, cards, notes and questions with a parent-stack picker, tags and "Create
  another"; stacks can sit inside other stacks (shown as a tree in the library, and run setup includes the stacks
  inside a ticked one); first launch shows a welcome page and a clickable in-app notification that opens the
  vault dialog, instead of a path field (TASK-0096 to 0099).
- A top bar (sidebar toggle, vault switcher, brand, global search, Create menu, notifications, help, settings,
  profile) and a foldable sidebar for the screens; global search over item titles; a notification list behind
  the bell (TASK-0093, 0094, 0095).
- Extensions: `studyfrog-ext` (importers and card types as plain data, a registry), an example extension
  (tab-separated importer, cloze card type) that changes no core crate, a card-type chooser and an "Import from
  a file" panel in the library, `docs/EXTENDING.md` and ADR 0008 (TASK-0063, 0064, 0065).
- Packaging: `scripts/package-appimage.sh`, a desktop entry and icon, and a CI workflow that runs the gate and
  builds the AppImage (TASK-0067). The 0.1.0 AppImage builds and starts locally; the workflow has not run yet.
- Choose the vault folder with the native dialog (through the desktop portal; the app binary supplies it, the
  UI crate does not link the dialog library) (TASK-0092).
- Shuffling is its own switch: any order, spaced repetition included, can be shuffled without changing which
  items are chosen or whether the schedule is updated (schema v7; runs stored as `shuffled` become sequential
  with the switch on) (TASK-0091).
- Open flags closed: answer runtimes are calibrated on a labelled set (word matching gets a pivot of 0.44;
  the language model keeps 0.7); typed-answer scores are stored with their run (schema v6) and shown in the
  review; `answer_engine = "words"` and a Settings choice force word matching; the user's easy/difficult mark
  suggests a rating; the fast/slow/minimum-words thresholds are configurable in `app.toml` (TASK-0086 to 0090).
- Reports: retention by time since the previous review, time to answer (percentiles, histogram, by the
  difficulty mark), and a 30-day review-load forecast with a backtest on real or simulated history; a Reports
  screen; each report computes in under 10 ms on 100,000 reviews (TASK-0059, 0060, 0061, 0062).
- Achievements: rules as data (built-in set or the vault's `achievements.toml`), statistics from the review
  history including streaks, unlocks remembered once (schema v5), an Achievements screen with progress and
  an announcement when something unlocks (TASK-0056, 0057, 0058).
- Reading speed can be measured on the Settings screen (read a lorem ipsum text, press done) and is kept in
  `app.toml`; rehearsal uses it as the baseline. Reading much slower than expected now suggests a harder
  rating (Good to Hard, Easy to Good) (TASK-0084, 0085).
- Rehearsal: mark an item Easy, Medium or Difficult while it is on screen (buttons or E/M/D); reading speed
  against a 238 words-per-minute baseline suggests a rating; the post-run review shows previous rating,
  own mark and rating and the suggestion, and can apply it, rebuilding the item's schedule (TASK-0081,
  0082, 0083).
- Semantic answer comparison: a candle MiniLM runtime (`studyfrog-candle`, linked by the app only) used
  automatically when `all-MiniLM-L6-v2` is installed under the app data directory, with fallback to word
  matching and the reason shown; the result line says which engine compared the answer. Measured: p95
  about 75 ms per comparison, about 85 MB of memory (TASK-0050, 0051).
- Answer comparison: `SlmRuntime` trait, scoring over embeddings (best of reference and accepted answers,
  rubric points, verdict bands, suggested rating), an in-house offline lexical runtime, property tests and a
  benchmark; typing an answer in rehearsal now shows the match, the verdict and a suggested rating
  (TASK-0049, 0052, 0053, 0054, 0055).
- Calendar screen: month grid with recurring sessions, day panel (edit, delete, skip a day, start a
  rehearsal for the session's stack) and a session form (TASK-0045).
- Board screen: columns with WIP counts and over-limit marking, drag and drop plus push/pull/up/down buttons,
  card editor with stack link, optional forcing past WIP limits (TASK-0048).
- Planning foundations: `studyfrog-calendar` (sessions, recurrence checked against a day-by-day oracle) and
  `studyfrog-board` (columns, WIP limits, strict/forced moves, push/pull, commands; property-tested); schema v4
  (sessions, board); service use cases for both; `core::civil` (dates, weekdays, month grid, local time); a
  local time-zone provider so daily limits and run previews use the local day (TASK-0044, 0046, 0047, 0078,
  0079, 0080).
- Link graph screen: the wikilink graph as a force-directed picture with pan, zoom, hover, click-to-open, a
  re-run button, an accessible item list, and live refresh that keeps node positions. New pure crate
  `studyfrog-graph` (SoA layout, uniform-grid repulsion, deterministic, zero allocation per step): 5k nodes /
  10k edges step in 4.0 ms; verified allocation-free with dhat. Items now show when they are next due
  (TASK-0042, TASK-0043). EPIC-009 and phase M4 complete.
- Editor: open any item from the browser, edit title, tags and text areas with a live Markdown preview (headings,
  emphasis, lists, task items, quotes, code, wikilinks and URL links), undo/redo, save with conflict detection
  (a file changed elsewhere is never overwritten), revert/reload, unsaved-changes guards, clickable wikilinks.
  `studyfrog-md::blocks` (preview model, pulldown-cmark), `studyfrog-service::document`, `resolve_title`
  (TASK-0040, TASK-0041).
- Rehearsal screen: keyboard-first (Space, 1-4, S, P, Esc), pause that hides the question, flip and type-answer
  modes, look-ahead content loading, end-of-run summary; every step saved through the service (spaced runs update
  the schedule, practice-only runs do not). Service `start_run`, `load_item_content`, `StepContext`; crashed runs
  are recovered when a vault opens. 15 tests (TASK-0039). EPIC-008 (UI shell) complete.
- Run setup screen: presets (spaced, everything in order, shuffled, hardest first, not seen for longest),
  all orders, filters (kind, stack, tags, state, difficulty), limit, requeue, and an explicit "update my
  schedule" switch; live count of matching items from the index. 6 tests (TASK-0073).
- Stack browser: choose a stack or the inbox, filter its items by title, create stacks and cards from the UI.
  Service use cases `create_stack`/`create_card` (never overwrite, validated, reindexed at once), in-house ULID
  composition with OS randomness, `list_items`; 14 new tests incl. a real-vault build-a-stack flow
  (TASK-0038).
- First desktop app (EPIC-008 so far): eframe/egui window with a vault chooser (remembers the last vault),
  library overview (stacks, counts, inbox, problems), navigation and settings view; pure
  `update(State, Msg) -> (State, Vec<Effect>)` with a worker-thread executor and 1 s change polling;
  21 tests incl. headless egui view tests and an accessibility audit; `--smoke-frames N` for CI
  (TASK-0036, TASK-0037).
- `studyfrog-service::session` (open a vault, index it, poll it), `studyfrog-store::overview` (stacks, counts,
  problems), per-user `app.toml` (`studyfrog-config`, `studyfrog-store::app_config`), `poll::prime`.
- `studyfrog-config` (ADR 0007, Proposed): pure parsing and rendering of `studyfrog.toml` (vault defaults:
  scheduler, retention, interval cap, FSRS weights) and `_stack.toml` (scheduler, retention, daily limits);
  unknown keys are errors with line numbers (TASK-0075).
- Settings pipeline: `_stack.toml` is indexed (`stack_identity`, `stack_config`, view `stack_setting`), loaded into
  the library, and included in polling; schema v3 (TASK-0075).
- SM-2 in the service: each item is scheduled by its stack's scheduler (default from `studyfrog.toml`),
  stack retention overrides apply to FSRS, switching a stack keeps items scheduled (TASK-0076).
- Per-stack daily new/review limits for spaced runs, counted from the review history with an injected day window;
  practice-only runs do not count (TASK-0077).
- Run filters are persisted with the run and round-trip exactly (TASK-0074). EPIC-018 complete.
- `studyfrog-service` (application layer, ADR 0006 Proposed): `apply_rating` (pure), `record_step`,
  `begin_run`, `recover_runs`, `new_run`/`candidates`; end-to-end tests over a real vault and database
  (spaced run updates the schedule; practice-only run leaves it untouched; due items return most-overdue
  first; crash recovery) (TASK-0033).
- `studyfrog-store`: schema v2 (run plan columns, in-place upgrade tested), run and event persistence
  with crash recovery, review and FSRS state storage, item positions (TASK-0072, TASK-0033).
- `studyfrog-core`: event name/payload encode and decode. Phase M3 complete.
- Queue builders in `studyfrog-rehearsal`: filters (kind, stack, tags case-insensitive incl. non-ASCII,
  state, difficulty), orders (spaced with daily budgets, sequential, seeded shuffle, hardest/easiest,
  stalest, lowest predicted recall), length cap; in-house SplitMix64 shuffle; 13 tests incl. an
  independent filter oracle; benchmark: engine step 23 ns, 10k-candidate queue 82 us to 3.9 ms
  (TASK-0034, 0069, 0070, 0035).
- Run plans in `studyfrog-core`: orders (due-first, sequential, shuffled, hardest/easiest, stalest,
  lowest predicted recall), filters, schedule effect (update vs practice-only), requeue-on-Again, limit
  (TASK-0068). Rehearsal is not only spaced repetition.
- `studyfrog-rehearsal`: pure run state machine (`step`), injected clock, latency excluding paused
  time, requeue with cap; 9 tests incl. an invariant property (TASK-0030, 0031, 0032, 0071).
- `studyfrog-store`: vault walker, in-house content hash, stack ownership by folder, non-fatal scanner
  with duplicate-id detection, `Library` builder (inbox stack for orphans), incremental reindex
  (stat-only for unchanged files, rename-safe, duplicate-healing), polling change detection with
  settle window; 17 tests and a 10k-note benchmark (cold index 0.57 s). EPIC-006 and phase M2
  complete (TASK-0027..0029).
- `studyfrog-store`: SQLite schema v1 (derived index + authoritative history), transactional
  migration runner, WAL/foreign-key pragmas, 6 tests (TASK-0025, TASK-0026).
- `studyfrog-md`: in-house ULID and RFC 3339 codecs, front-matter fence/key-block splitter, typed
  parser and renderer for stack/note/card/question with verbatim unknown keys, `???` splitter,
  wikilink extraction, duplicate-id check; 25 tests incl. 1000-case round-trip property
  (TASK-0020..0024). EPIC-005 complete.
- `studyfrog-srs`: SM-2 (`sm2_next`, ease/interval rules from the original), `Sm2State` in core, and a
  criterion benchmark (100k cards: FSRS 2.9 ms, SM-2 0.72 ms); FSRS budget tightened to 10 ms
  (TASK-0018, TASK-0019). EPIC-004 and phase M1 complete.
- `studyfrog-srs`: FSRS-6 (`initial_state`, `next_state`, `retrievability`, intervals, Anki-style
  deterministic fuzz); `MemState` in core; golden test reproducing the py-fsrs reference intervals and
  7 property tests (TASK-0016, TASK-0017).
- `studyfrog-core`: `Run`, `RunMode`, `RunState`, `RunEvent`, `EventKind`, `Latency` and persisted-name
  functions, 2 tests (TASK-0015). EPIC-003 (core domain types) complete.
- `studyfrog-core`: `Rating` (persisted values 1-4), `Review`, `rating_value`/`rating_from_value`,
  3 tests (TASK-0014).
- `studyfrog-core`: `Meta`, `Stack`, `Note`, `Card`, `Question`, `UnixMs` and `Library` (one arena per
  kind, no `Item` enum), plus a test (TASK-0013).
- `BOARD.md`: Kanban view generated from issue front-matter by `scripts/board.sh`; WIP limit 3;
  `scripts/check.sh` fails when it is stale.
- `CHANGELOG.md` and `CLAUDE.md` (working rules: keep board and changelog current).
- `studyfrog-core`: `Uid`, typed arena index `Idx<T>`, append-only `Arena<T>` with free
  functions, 7 tests (TASK-0012).
- M1 issues: EPIC-003/004, STORY-0005..0008, TASK-0012..0019.
- `tools/repo-lint` (standalone, `syn` + `serde-saphyr`): inherent-`impl` scan including macro
  bodies, and issue front-matter validation (TASK-0006).
- `scripts/check-issues.sh`, `scripts/check-deps.sh`, `scripts/check-impls.sh`,
  `scripts/linecount.sh`, `scripts/check.sh` (TASK-0003..0005).
- Documentation library: vision, architecture, design principles, glossary, ADRs 0001-0005,
  roadmap M0-M8 with phase files, specs, research, contributing guides (TASK-0007..0011).
- M0 issues: EPIC-001/002, STORY-0001..0004, TASK-0001..0011.
- Cargo workspace with 12 documented, empty crates and shared lints (TASK-0001, TASK-0002).
- `rustfmt.toml`, `clippy.toml`, `rust-toolchain.toml` (1.99.1), `.gitignore`.

### Changed
- The Create button opens the dialog (the dropdown with study session, board card and import entries is gone;
  those are made on their own screens). The inline "new stack" form in the library is replaced by "New stack…".
- The screen buttons moved from the top row to a sidebar; "Close vault" moved into the profile menu.
- The lowest-recall queue order is 7 times faster (recall computed once per item) (TASK-0066).
- The application binary is called `studyfrog`.
- Dev builds use line-table debug info only, so the many test binaries link much faster.
- `md`'s RFC 3339 codec shares `core::civil` instead of carrying its own date arithmetic.
- Daily limits and run creation use the local day (was the UTC day).
- Queues are built from the index (`studyfrog_store::overview::load_indexed_items`) instead of a parsed
  `Library`; `new_run` no longer takes a library.
- Stack settings (`daily_*`, `srs`) are no longer `_stack.md` front-matter; they moved to `_stack.toml`
  (ADR 0007). `StackDoc` now carries only identity, description and body; core `Stack` has `settings`.
- ADR 0005 amendment accepted: no `gray_matter`; in-house fence splitter plus `serde-saphyr`
  (TASK-0021 retitled). New design principle P10: build instead of buy when we can do better.
- ADRs 0002-0005 accepted (GUI: egui/eframe; persistence: rusqlite + Markdown files;
  SLM: candle, gated on M6 spike; Markdown schema).
- ADR 0005: YAML crate decided (`serde-saphyr`); amendment to drop `gray_matter` is Proposed.
- `docs/specs/srs-algorithm.md` pins FSRS-6 with default weights.
- Research citations verified where possible and marked `[v]`/`[u]`; benchmark budgets
  documented as provisional with rationale.
- Default branch is `main`.

### Removed
- Placeholder hello-world package and the unverified `repository` manifest field.

### Fixed
- Graph layout never came to rest because the repulsion jumped to zero at its cutoff (nodes flickered in and out of
  range); a smooth window fixes it. Found by measuring energy, not by a failing test.
- Board/issue bookkeeping: TASK-0016, STORY-0007 and EPIC-004 were still `backlog` after M1. `repo-lint`
  now rejects a parent whose children are all done but is not, a done parent with open children, and a
  done item with unchecked acceptance criteria.
- `scripts/linecount.sh` no longer miscounts the `wc` total line on non-English locales.

[Unreleased]: https://github.com/louisgoodnews/study-frog/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/louisgoodnews/study-frog/compare/v0.9.0...v1.0.0
[0.9.0]: https://github.com/louisgoodnews/study-frog/compare/v0.8.0...v0.9.0
