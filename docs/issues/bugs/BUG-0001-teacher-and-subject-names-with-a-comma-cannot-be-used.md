---
id: BUG-0001
type: bug
title: "Teacher and subject names with a comma cannot be used"
status: done
epic: EPIC-023
parents: []
estimate: 3
labels: [m10, ui, config]
acceptance_criteria:
  - "A name that cannot be stored (empty, with a line break or a duplicate) is refused before anything is saved, with a message that says why next to the field, and the typed text stays in the field"
  - "A teacher named 'Prof. Dr. Künkelmann, Harry' can be added, assigned to a subject, saved and loaded again, and names with commas work in stack and item subject lists"
  - "The error text says 'load' when a load failed and 'save' when a save failed"
---

## Context
The owner saw "Could not save the subjects: a teacher's name must be one line without commas" in the notification list on Home (screenshot, 2026-11-02) after adding teachers and stacks.

## Description
On the Settings screen, typing a teacher or subject name that contains a comma (for example the common German form "Müller, Hans") and pressing Add shows nothing wrong at first. The name is cleared from the field, the save is attempted, and only then a notice appears in the bell list: "Could not save the subjects: a teacher's name must be one line without commas". The notice is easy to miss, the name has to be typed again, and nothing at the field says that commas are not allowed.

Reproduced on 2026-11-02 with a headless test (removed again): `TeacherDraft("Müller, Hans")` then `AddTeacher` returns `SaveSubjects` with that name, clears the draft and leaves `notice` empty; the book check (`studyfrog_config::subjects::book_problem`) then refuses it, and the `Loaded(Err)` handler shows the notice above with the book unchanged.

## Acceptance Criteria
- [x] A name that cannot be stored (empty, with a line break or a duplicate) is refused before anything is saved, with a message that says why next to the field, and the typed text stays in the field
- [x] A teacher named 'Prof. Dr. Künkelmann, Harry' can be added, assigned to a subject, saved and loaded again, and names with commas work in stack and item subject lists
- [x] The error text says 'load' when a load failed and 'save' when a save failed

## Design Notes
Cause: `SubjectsMsg::AddTeacher` and `AddSubject` in `crates/studyfrog-ui/src/subjects_ui.rs` only check for an empty name; the one-line, no-comma rule lives in `book_problem` (`crates/studyfrog-config/src/subjects.rs`) and is applied only in `save_subjects` (service). The rule exists because a stack's `subjects` key is a comma-separated list (docs/specs/markdown-schema.md). `SubjectsMsg::Loaded(Err)` always says "Could not save", even though the message is also used for other failures.
Fix: run `book_problem` on the changed book inside `update` before emitting `SaveSubjects`, keep the draft and show the reason under the field. Separate the load and save texts. A regression test covers a name with a comma, a name with a line break and a duplicate.
Decision (owner, 2026-11-02): names with commas must be allowed; professors such as 'Prof. Dr. Künkelmann, Harry' cannot be assigned to a subject today. That needs a schema change: a stack's and an item's `subjects` key is written as a comma-separated list (`edit_form.rs` joins with ", ", `studyfrog-store/src/overview.rs` `subjects_of` splits on commas, and the index stores the same text), so a name with a comma becomes two names. Proposed format: names containing a comma, a double quote or a leading or trailing space are written in double quotes with backslash escapes, and the reader splits on commas outside quotes; plain lists written by earlier versions read the same as before, so no migration is needed. `book_problem` then refuses only empty names, line breaks and duplicates. Update docs/specs/markdown-schema.md and the round-trip property test with names that contain commas and quotes.

Fixed on 2026-11-02 as proposed: `studyfrog_md::names` writes a list that has a name with a comma, quote, backslash or bracket (or blanks at its ends) as a flow sequence of double-quoted names, and reads it back; plain lists read and write exactly as before, so there is no migration. The store index keeps the stored value and splits it with the same reader (`overview.rs`), the item page reads and writes it through `extra_names` and `with_extra_names`, and `book_problem` now refuses only empty names and control characters. The Settings screen checks a new teacher or subject before saving, keeps the typed text and shows the reason under the field (`teacher_error`, `subject_error`); the save error says 'Could not save the subjects'. A shared `without_unknown_teachers` keeps that check and the save in step. Not seen in a real window: the screen was tested headless, like the rest of the settings.

## Dependencies
None.

## Open Questions
None. Commas are allowed; the format above is a proposal for the owner to veto.
