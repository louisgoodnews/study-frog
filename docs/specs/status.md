# Item status

Every stack, note, card and question has a status, in the manner of an issue in Jira: it says where the item stands.
Planned for 0.3.0 (STORY-0061). Names accepted by the owner on 2026-11-02.

## The statuses
| Stored value | Label shown | Meaning | In the schedule |
|---|---|---|---|
| `new` | Not studied yet | Never rehearsed (a note: never viewed). The default. | Scheduled |
| `learning` | Learning | Rehearsed, not yet known well. | Scheduled |
| `reviewing` | In review | Known well; kept fresh by spaced repetition. | Scheduled |
| `done` | Complete | The user is finished with it. | **Skipped** |
| `paused` | On hold | Kept, but not studied for now. | **Skipped** |

Stored values are English and stable (they are written to files); only the labels are translated or reworded.

## Where it is stored
A one-line front-matter key `status` on every kind, next to `priority` and `difficulty`
([markdown-schema](markdown-schema.md)). The key is **optional**:

- **Absent:** the effective status is derived from the history. No review yet (a note: no view yet) is `new`; at least one
  review (a note: one view) is `learning`.
- **Present:** it wins over the derived value, whatever the history says.

The app writes the key when the user sets a status and when the first review moves an item out of `new`
(`new` is then not written; absence means derived). An unknown value in a file is kept verbatim, shown as the derived
status, and flagged in the item's details; it is never overwritten silently.

## Who may change it
| Change | By |
|---|---|
| any status to any other | the user, on the item page, in the list (bulk too) and on the board of a stack |
| `new` to `learning` | automatic, on the first review (a note: the first view), only while the status is absent or `new` |
| `learning` to `reviewing`, `reviewing` to `learning` | **suggested** from familiarity (STORY-0058), never automatic |
| to `done` | the user only; a suggestion may be offered when an item stays near full familiarity (see below) |

Automatic changes never touch `reviewing`, `done` or `paused`. A suggestion is a chip next to the status ("Looks familiar:
mark In review?") that the user accepts or dismisses; a dismissed suggestion is not repeated for the same item for 30 days.
Proposed thresholds, to be set in Settings: familiarity 0.8 or more after at least 3 reviews suggests `reviewing`;
below 0.5 suggests `learning`; 0.95 or more for 30 days suggests `done`.

## Stacks
A stack has its own status (the same five values), and shows a roll-up beside it: the number of items in each status,
counting the items of its sub-stacks. The roll-up is derived and not stored. When every item is `done`, the stack offers
"mark Complete" as a suggestion.

**Inheritance:** an item is skipped by the schedule when it or any stack above it is `done` or `paused`. Its own status is
not changed by that.

## What the status does
- **Calendar and due queues** skip `done` and `paused` items and stacks (STORY-0062); they resume when the status changes back.
- **Run filters** get a status filter; runs that ignore the schedule (every order except `Due`) include all statuses
  unless the filter says otherwise.
- **Familiarity** stops falling without contact while an item is `done` (STORY-0058).
- **Reports** count every status; the stack Summary shows a status ring chart (STORY-0063).
- **Library list, query and Backlog tab** have a `status` column and field (`status IN (new, learning)`).
- **Import:** an Anki card with history becomes `learning`; everything else is left without the key.

## Boards
The global kanban board (EPIC-011) keeps its free-form columns. The Board tab of a stack groups its items by status, and
moving a card between columns changes the item's status. (Accepted by the owner on 2026-11-02: only a stack's Board tab is
bound to statuses, the global board stays free-form.)

## Tests
Round trip of the key for all four kinds; absent key derives `new` and `learning`; unknown value kept; automatic change
happens once and never overrides a set status; inheritance from a stack; the schedule skips `done` and `paused`.
