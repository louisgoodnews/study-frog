# Calendar

The calendar plans your studying: study sessions, tasks, appointments, exams and deadlines, and the reviews your
schedule has planned. Open **Calendar** in the sidebar. Each [[stack-page|stack page]] also has a Calendar tab that
shows only that stack.

## The month

The month fills the window. Entries are coloured by kind; a day with many entries shows "+ n more". Use **Previous
month**, **Next month** and **Today** to move. Click a day to see it in the side panel.

## Adding an entry

1. Click a day, then press **Add to calendar**.
2. Choose the kind: **study** (a study session), **task**, **appointment**, **exam** or **deadline**.
3. Fill in **Title**, **Date (YYYY-MM-DD)**, **Time (HH:MM)** and **Duration (minutes)**. The date field has a
   month grid to pick from.
4. Optionally link a **Stack** and write **Notes**.
5. **Repeat**: Does not repeat, Daily, Weekly (choose the days) or Monthly, every so many days, weeks or months.
   It ends never, on a date, or after a count.
6. **Remind me**: at the time, 15 minutes before, 1 hour before, 1 day before or 1 week before (up to five).
7. Press **Save entry**.

A stack's due date, set in the [[create-dialog|Create dialog]] or on its stack page, appears as a deadline.

## The day panel

**Sessions on** the chosen day lists its entries. For each you can:

- **Edit** or **Delete** it,
- **Skip** a repeating entry for that day only,
- **Start rehearsal for** it, which opens the run setup with its stack. See [[rehearse]].

**Coming up** lists what is planned in the next two weeks.

## Planned reviews

The calendar also shows the reviews your schedule has planned, as chips like "French: 4 of 10 due": the stack, the
number of items due that day, and the size of the stack. They look different from your own entries. Items and stacks
that are Complete or On hold make no chips. See [[statuses|Item statuses]].

- Click a chip to open its stack.
- **Move the reviews to another day**: drag the chip to a day. If you prefer not to drag, use **Move to another
  day…** in the chip's menu or in the side panel, choose the date and press **Move**.
- A move changes only the due date of those items. If the new day is after an exam or due date of their stack,
  StudyFrog says so.
- **Undo last move** takes the last move back.
- Untick **Show the reviews the schedule plans** to hide the chips. This is remembered in `app.toml` as
  `calendar_reviews = false`.

## Reminders

A reminder shows once, as a short message and as a notification under the bell, while StudyFrog is open. Clicking
the notification opens the calendar. StudyFrog does not remind you while it is closed; a missed reminder shows
once when you open it again, if its entry has not ended yet. See [[notifications]].

## Related pages

[[home]] shows today's sessions and the next seven days. [[board]] holds tasks. See also
[[getting-started|Getting started]] and the [[faq]].
