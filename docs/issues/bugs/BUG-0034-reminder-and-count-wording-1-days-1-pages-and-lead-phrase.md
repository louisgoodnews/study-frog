---
id: BUG-0034
type: bug
title: "Singular counts read '1 days' and '1 pages', and reminders say 'in 2 h' for an item due in 1 h"
status: done
estimate: 1
labels: [m14, calendar, ui]
acceptance_criteria:
  - "A count of one reads in the singular: '1 day' for the streak, '1 page' for the PDF find count"
  - "A homework reminder states the time left before the due time, or says it is a lead time, so a due time one hour away is not read as two hours away"
  - "The homework reminder toast and the notification line use the same wording"
---

## Context
Found while auditing STORY-0166, STORY-0169 and STORY-0170 in the running app (2026-11-22):
- Home shows the streak as "1 days".
- The PDF viewer's find result reads "1 pages" for one page with a hit.
- The homework reminder for "Lektuere Kapitel 4", due in 11 h, reads "Lektuere Kapitel 4 (in 1 days, 2026-11-23 09:00)": the 24 h lead is shown as "1 days".
- For "Kurzquiz", due 22:30 with about 68 minutes left, the toast and notification read "(in 2 h, 2026-11-22 22:30)". The 2 h is the lead time that began, not the time left, so the wording suggests more time than there is.

Screenshots: `/tmp/claude-0/audit/g5/0166-homework-all.png` (toast "in 1 days"), `/tmp/claude-0/audit/g5/0166-day-kurzquiz.png` (toast "in 2 h" for an item due the same day), `/tmp/claude-0/audit/g5/0170-pdf.png` (find result "1 pages"), `/tmp/claude-0/audit/g5/0169-tip-vault.png` (Home streak "1 days").

## Description
Use a singular noun for a count of one in the streak, the PDF find count and the reminder text. In the reminder, either show the time left ("due in 1 h") or name the lead ("24 h before"), and make the toast and the notification say the same. Add a test for a count of one and for a reminder with less time left than its lead.

## Acceptance Criteria
- [x] A count of one reads in the singular: '1 day' for the streak, '1 page' for the PDF find count
- [x] A homework reminder states the time left before the due time, or says it is a lead time, so a due time one hour away is not read as two hours away
- [x] The homework reminder toast and the notification line use the same wording

## Design Notes
- Fixed on 2026-11-23. A shared count helper, `studyfrog_core::count` (`noun` and `counted`), gives the singular for one; the "1 run" line of the learn-mode report uses it too (its private copy is removed). Reminders: `DueHomeworkReminder` now carries `remaining_min`, and the homework text states the time left before the due time ("Kurzquiz (due in 1 h 8 min, 2026-11-23 22:30)") instead of the lead it fired at; the toast and the notification both use `reminder_ui::homework_text`. Session reminders keep their lead wording with the singular ("in 1 day"). Also singular now: the Home streak (`view/home.rs`), the PDF find count (`view/pdf.rs`), the library page count (`view/library_pdfs.rs`) and the CSV retention bucket label (`studyfrog-service/src/report_csv.rs`). Before: `/tmp/claude-0/layout/before-day-toasts-1280.png` ("Kurzquiz (in 1 days, ...)", "Essay 2 (in 1 days, ...)"). After: `/tmp/claude-0/layout/after4-day-toasts-1280.png` ("Extra quiz (due in 15 h 13 min, ...)", "Mini Test (due in 17 h 13 min, ...)"). Tests: `studyfrog-ui/tests/reminders_homework.rs::a_reminder_states_the_time_left_not_the_lead_it_fired_at`, `reminders.rs` (expects "in 1 day"), `home.rs::a_streak_of_one_day_reads_in_the_singular`, `studyfrog-core` `count::tests::one_takes_the_singular_and_every_other_count_the_plural`, and `studyfrog-service/tests/report_csv.rs` ("retention,1 day"). Not seen in the running app: the streak and the PDF find count (the test vault has no streak and no PDF); both are covered by the tests above and the call sites.
- Touchpoints: `crates/studyfrog-calendar` (reminder text), `crates/studyfrog-ui/src/reminder_ui.rs`, the streak view on Home, `crates/studyfrog-ui/src/view/pdf.rs` (find count).
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0166
- EPIC-056

## Open Questions
- None.
