# ADR 0011: Stack folders, references and memory per stack

- Status: **Accepted** (2026-11-03, the owner's decisions; the wording is Claude's and the owner may amend it)
- Date: 2026-11-03

## Context
A vault is a folder of Markdown files. Today every folder with a `_stack.md` is a stack, an item belongs to the one stack
whose folder it sits in (loose items belong to the Inbox), and the schedule, the reviews and the familiarity of an item are
kept once per item. The owner asked for (2026-11-03): an item can be referenced by several stacks; a folder is a stack
only when marked; the vault is viewed like Obsidian's file explorer; each stack has its own independent memory of an item.
The decisions are in TASK-0210; this record states the rules the later tasks (TASK-0211 to TASK-0220, TASK-0202 to
TASK-0204) implement.

## Decision

### Stacks and folders
1. A folder is a **stack** when it holds a stack file (`_stack.md`, as today) and a **normal folder** otherwise. Nothing
   changes in existing vaults. The stack file is the mark, so unmarking a stack deletes only that file (its content is kept
   in the undo snapshot) and marking writes one with a new id and the folder's name as the title.
2. The **primary stack** of an item is the nearest stack folder above its file; none means the Inbox (uid 0). A normal
   folder inside a stack folder belongs to that stack. Moving a file across a stack boundary changes its primary stack.
3. `referenced_by` in the item's front matter lists the **other** stacks that reference it, by stack id (a ULID), so the
   list survives renames and moves of the folders. It never lists the primary stack and never lists a stack twice. Unknown
   keys stay as they are; vaults without the key read exactly as before.

### Memory per stack
4. **Memory, schedule, due date, familiarity and difficulty are independent per (item, stack)** for every stack that holds
   the item (its primary stack and each referencing stack), each with that stack's own scheduler, retention, limits and
   weights. They live in the database (`mem_state`, `sm2_state`, `review` and the familiarity inputs gain a `stack`
   column) and are **never written into the card's file**, which holds only `referenced_by`.
5. The item's **overall value** is the average over its references: familiarity and difficulty (the marks easy, medium and
   difficult count as 1, 2 and 3 and the average shows the nearest word with the number). The due date has no average:
   each stack has its own.
6. **Runs.** A run over several stacks queues a card once and counts it **due if it is due in any selected stack**. One
   rating writes **one review for every selected stack that holds the card**, also where it was not yet due (an early
   review is scheduled like any early review), and none for a stack outside the selection. A difficulty mark given in a run
   goes to the same stacks. A run of one stack looks at that stack's memory only. Daily limits and the planner of a stack
   look only at that stack's schedule for its items.
7. **Existing data** becomes the primary stack's in a schema migration that can be checked on a copy of the database.

### Shape shown to the user and in exports
8. The item page, the library and the exports show, per reference, `{ source, difficulty, familiarity }` plus the due date
   and the stability, then the average. `source` is the stack (`primary` or `referenced`); `difficulty` is the last mark
   given in that stack, else the item's own mark. The library shows the primary stack, an "Also in" column and the average
   with the per-stack values in the tooltip.

### What happens to references and to per-stack data
9. **Moving files.** Moving a file within a stack changes nothing. Moving it across a stack boundary changes the primary
   stack (the plan says how many items change and asks first), `referenced_by` is fixed so it never lists the new primary
   stack, and the per-stack data stays with the stack: the old primary stack keeps its memory only if the item is still
   referenced by it, otherwise that data is dropped with a count in the plan.
10. **Deleting a stack** (or unmarking it as a stack) removes it from the `referenced_by` of the items that listed it
    (with a count), drops its per-stack data and moves its own items to the stack above, or to the Inbox. A reference to a
    stack that no longer exists (found by the vault check) is a vault problem with one action that removes it.
11. **Import and export.** JSON, TOML and RON carry references (`referenced_by` as stack folders) and, in the advanced
    mode, the per-stack values with the average and the review history; CSV gets an optional `also_in` column (stack
    folders separated by semicolons); an Obsidian vault maps its folders to normal folders unless they hold a stack file;
    an Anki deck maps to a stack as today. TOML and RON are exports and are read back where a reader exists.

## Options considered
- *Per-item memory only, references as tags:* no migration, but a card studied in two stacks would share one schedule,
  which the owner rejected (a stack's exam and settings are its own).
- *Copies of the card per stack:* independent memory for free, but edits diverge. Rejected.
- *The mark as a key in the folder's settings file instead of `_stack.md`:* a second place to keep in step. Rejected; the
  stack file already is the mark.

## Consequences
- The index gets a membership table (stack, item, primary or referenced); the queue builders, the summary, the
  roadmap, the reports and the planner replace their folder-prefix checks with it (TASK-0211, TASK-0212).
- `schedule` and `reviews` change their key and a migration moves existing rows to the primary stack (TASK-0218); the pure
  schedulers in `studyfrog-srs` do not change.
- Familiarity is computed per stack from that stack's reviews and exam date (TASK-0219).
- More to maintain, and a card studied in several stacks produces several reviews per answer. The size of the reviews
  table grows by the number of references per card, which stays small in practice.
