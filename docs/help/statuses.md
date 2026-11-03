# Item statuses

Every stack, note, card and question has a status. It says where the item stands, like the status of a task in a
project tool.

## The five statuses

- **Not studied yet**: never rehearsed (a note: never viewed). This is where every item starts. It is scheduled.
- **Learning**: rehearsed, but not yet known well. It is scheduled.
- **In review**: known well, and kept fresh by spaced repetition. It is scheduled.
- **Complete**: you are finished with it. It is **not** scheduled.
- **On hold**: kept, but not studied for now. It is **not** scheduled.

In files and in queries the statuses are written `new`, `learning`, `reviewing`, `done` and `paused`.

## Automatic or set by you

An item's status menu has a sixth choice: **Automatic**. With Automatic, StudyFrog works the status out from your
history: an item with no review yet (a note: no view yet) is Not studied yet; after the first review (or view) it is
Learning. The menu shows the status it worked out, for example "Automatic (Learning)".

When you choose a status yourself, it is written into the item's file as `status: done` (for example) and wins over
the history. Choosing Automatic again removes it. StudyFrog never moves an item to In review, Complete or On hold by
itself: those are your decisions.

## Where to change it

- On the [[item-page|item page]], in the **Status** menu.
- In the [[library]] list: select items and use **Set status** to change many at once.
- On a [[stack-page|stack page]]: the status menu in the header changes the stack's own status. On the **Board**
  tab, drag an item to another column. On the **Backlog** tab, moving an item to **Up next** makes it Learning and
  moving it to **Not yet planned** puts it On hold.

## Stacks

A stack has its own status, with the same five values. Next to it, the stack page shows how many of its items, the
stacks inside included, are in each status.

When a stack is **Complete** or **On hold**, everything inside it is skipped by the schedule too, without changing
the items' own statuses. Set the stack back, and its items come back.

## What a status does

- **Rehearsal and the calendar**: Complete and On hold items and stacks are left out of spaced runs and of the
  planned reviews in the [[calendar]]. Runs that go through everything (other orders) include all statuses.
- **Familiarity** stops falling while an item is Complete. See [[familiarity]].
- **Reports and summaries** count every status; a stack's Summary tab shows them as a ring.
- **Search**: the `status` column in the library and the `status` field in the [[query-language|query language]],
  for example `status IN (new, learning)`.
- **Import**: an imported card whose review history you bring along shows as Learning, because it has reviews.
  See [[import-export]].

## A status StudyFrog does not know

If a file has a status value StudyFrog does not know (a typo, say), it is kept in the file as it is, and the item
shows the status worked out from its history. StudyFrog never overwrites it silently; fix it in the file or choose a
status in the menu.

## The two boards

Only a stack's **Board** tab is tied to statuses. The [[board|Board]] screen in the sidebar is a free-form task board.

See also the [[glossary]] and the [[faq]].
