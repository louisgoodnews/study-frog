# Glossary

The words StudyFrog uses, in alphabetical order.

**Achievement**: a milestone, such as a week of reviews in a row. Some unlock once for everything, some separately
for each stack. See [[achievements]].

**Active time**: time that counts on a clock in StudyFrog. Paused time is left out; on the item page a minute
without input stops the clock.

**Answer comparison**: how StudyFrog compares a typed answer with the reference answer, as a match in percent and a
verdict (Correct, Partly right, Not quite). It measures similarity, not truth. See [[rehearse]].

**Assistant**: an optional AI model you connect. It proposes notes, cards, questions and edits, and can examine you.
See [[assistant]].

**Backup**: one `.tar` file with your vault's files, its history and, if you want, your settings and keys. Restored
into a new or empty folder. See [[settings]].

**Board**: a kanban board of task cards in columns. The Board screen is free-form; a stack's Board tab sorts its
items by status. See [[board]].

**Card**: a flashcard with a front and a back. You flip it while rehearsing.

**Countdown**, **Count up**: the two clocks a run can have. A countdown ends the run when the time is up; a count up
shows the time used. See [[rehearse]].

**Difficulty**: easy, medium or difficult, set by you on an item or stack. FSRS also has its own memory difficulty
for every reviewed item, on a scale of 1 to 10.

**Due**: an item is due when the schedule says it is time to review it. A stack can also have a **due date**, a
deadline on the calendar.

**Export**, **Import**: writing a part of your library to a file, and reading one in, with a preview. See
[[import-export]].

**Familiarity**: how familiar you are with an item, from 0 to 100 %. It grows with good ratings and views and falls
with bad ratings and time. See [[familiarity]].

**Front matter**: the block of keys at the top of a Markdown file, between two `---` lines, with the item's id, kind,
title, tags, status and so on.

**FSRS**: the default scheduling method. It models memory with stability, difficulty and retrievability.

**Help page**: a page of this help. Supplied pages come with StudyFrog and are read-only; **your pages** are Markdown
files in the vault's `wiki/` folder. See [[help]].

**Inbox**: the items that are not in any stack.

**Item**: anything you can study: a note, a card or a question. Stacks are shown on item pages too.

**Library**: the list of all your items, with search, sorting and bulk changes. See [[library]].

**Mark**: easy, medium or difficult, given to an item during a run (keys E, M, D). It records how the item felt and
never changes the schedule by itself.

**Mastered**: an item whose next review is 21 days or more away.

**Note**: free Markdown text. It can link to other items. Its views count for familiarity.

**Notification**: a message kept under the bell in the top bar. See [[notifications]].

**Pacing**: whether a run has no clock, a countdown or a count up.

**Planned review**: a chip on the calendar showing how many items of a stack the schedule makes due on a day. It can
be moved to another day. See [[calendar]].

**Practice only**: a run whose ratings are recorded but do not change the schedule.

**Priority**: lowest, low, medium, high or highest. It decides what goes first in spaced runs.

**Profile**: a set of personal settings, recent vaults and API keys. Several people or roles can each have one.

**Query**: a typed search in the library, such as `type = card AND due <= 7d`. See [[query-language]].

**Question**: a prompt with a reference answer, answered by typing. StudyFrog compares your answer with the
reference.

**Rating**: Again, Hard, Good or Easy: how well you recalled an item. It feeds the schedule.

**Reading speed**: how many words a minute you read. StudyFrog measures it in Settings and uses it to suggest
ratings and to weigh familiarity.

**Retention**: the share of reviews you recall. The **target retention** of FSRS (default 0.9) is the recall the
schedule aims for.

**Retrievability**: the chance, as FSRS estimates it, that you recall an item right now.

**Review**: one recorded recall of an item with a rating. Reviews make up your history.

**Run**: one rehearsal session over a list of items. The **run plan** says which items, in which order, with which
clock, and whether the schedule changes. See [[rehearse]].

**Saved filter**: a library search kept under a name in `filters.toml`.

**Session**: a planned block of study time on the calendar, usually linked to a stack. Other calendar entries are
tasks, appointments, exams and deadlines.

**Settings file**: `studyfrog.toml` (vault defaults) or `_stack.toml` (one stack). Plain text you edit yourself. See
[[settings]].

**SM-2**: a simpler, classic scheduling method you can choose instead of FSRS.

**Source**: the PDF page a note, card or question was made from. Shown as a link.

**Spaced repetition**: reviewing just before you would forget, at growing intervals.

**Stability**: in FSRS, how long your memory of an item lasts.

**Stack**: a container of items, like a deck or a notebook. It is a folder in your vault with a `_stack.md` file.
Stacks can sit inside stacks (**sub-stacks**). See [[stack-page]].

**Status**: where an item stands: Not studied yet, Learning, In review, Complete or On hold. See [[statuses]].

**Streak**: the number of days in a row on which you reviewed something.

**Subject**, **Teacher**: subjects are given to stacks and passed on to what is inside; a subject can have a teacher.
Managed in [[settings]].

**Tag**: a free word on an item, for searching and filtering.

**Tutorial**: a list of steps in the app that opens each screen it describes. **First steps** is offered on the first
run. See [[help]].

**Vault**: the folder holding all your Markdown files. Everything StudyFrog knows starts there. See
[[getting-started]].

**Wikilink**: a link between items, written `[[Title]]`. See [[graph]].

**WIP limit**: the most cards a board column should hold at once ("work in progress").
