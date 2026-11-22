---
id: BUG-0034
type: bug
title: "Singular counts read '1 days' and '1 pages', and reminders say 'in 2 h' for an item due in 1 h"
status: backlog
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
- [ ] A count of one reads in the singular: '1 day' for the streak, '1 page' for the PDF find count
- [ ] A homework reminder states the time left before the due time, or says it is a lead time, so a due time one hour away is not read as two hours away
- [ ] The homework reminder toast and the notification line use the same wording

## Design Notes
- Touchpoints: `crates/studyfrog-calendar` (reminder text), `crates/studyfrog-ui/src/reminder_ui.rs`, the streak view on Home, `crates/studyfrog-ui/src/view/pdf.rs` (find count).
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0166
- EPIC-056

## Open Questions
- None.
