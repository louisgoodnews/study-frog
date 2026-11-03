# The query language

The [[library]] search has a **Query** mode. In it you type a query, a little like SQL or Jira's search, to find
exactly the items you want and sort them. This page lists everything a query can say.

## Examples

```
type = card AND status IN (new, learning) AND familiarity < 0.4
stack = "French/Verbs" AND due <= 7d ORDER BY due
tag = exam AND NOT status = done
subject = Maths OR subject = Physics ORDER BY familiarity ASC, title
text ~ "Prüfung" AND updated >= -14d
priority >= high AND difficulty IS EMPTY
status IN (done, paused) AND stack = French
```

## The parts of a query

A query is one or more **conditions**, joined with `AND` and `OR`, optionally followed by `ORDER BY`.

- `field = value`, `!=`, `<`, `<=`, `>`, `>=`: compare.
- `field IN (a, b, c)` and `field NOT IN (a, b)`: one of a list.
- `field IS EMPTY` and `field IS NOT EMPTY`: the field has no value.
- `field ~ "words"` and `field !~ "words"`: contains, or does not contain, these words (not case-sensitive).
- `NOT` turns a condition around. Brackets group conditions: `(a OR b) AND c`.
- `ORDER BY field ASC` or `DESC`, up to three fields separated by commas.

Keywords and field names may be written in any case. An empty query shows every item.

## Values

- A **word**: letters, digits, `-`, `_`, `.` and `/`, for example `card` or `French/Verbs`.
- Anything with spaces, commas or brackets goes in double quotes: `"Prof. Dr. Smith"`.
- A **number**: `0.4`. A **percent**: `40%` is the same as `0.4`.
- A **date**: `2026-11-18` or `today`.
- A **relative date**: `7d` is seven days from today, `-14d` fourteen days ago; also `2w` (weeks), `1m` (months) and
  `1y` (years).

## Fields

- `text`: title and body together. Use `~` or `!~`.
- `title`: `=` is the whole title; `~` part of it.
- `type`: `note`, `card` or `question`.
- `status`: `new`, `learning`, `reviewing`, `done`, `paused`. See [[statuses|Item statuses]].
- `stack`: a stack's folder or title; includes the stacks inside it.
- `subject`, `teacher`: the subjects of the item's stack and their teachers.
- `tag`: true when the item has the tag.
- `priority`: `lowest` < `low` < `medium` < `high` < `highest`. Its own or its stack's.
- `difficulty`: `easy` < `medium` < `difficult`, if any.
- `familiarity`: from 0 to 1 (or a percent). Items without history have none. See [[familiarity]].
- `due`: the date of the next review; empty for an item never rehearsed.
- `reviews`: how many reviews the item has.
- `created`, `updated`: dates.
- `path`: the file's path in the vault.

A field that can be missing (difficulty, familiarity, due, created, updated) matches no comparison when it is
missing, except `!=`. Use `IS EMPTY` to find those items.

## Mistakes

A query with a mistake is not run. The message under the box says what is wrong and in which column (counting
characters from 1), and the box marks the place. For example:

- `type =` gives "Expected a value after `=` at column 7".
- `status = nope` lists the statuses you can use.
- `tga = exam` asks "Did you mean `tag`?".
- `type > card` says that `>` does not apply to `type`.

While you type, field names, operators and values (statuses, types, stacks, subjects, tags) are suggested.

## Standard mode and saved filters

The Standard mode's fields are a query too. Switching to Query shows it, and a simple query (only `AND` of simple
conditions) switches back into fields.

Save a query under a name with **Filters**, **Save current filter**. Saved filters live in `filters.toml` in the
vault:

```toml
[[filter]]
name = "Hard cards"
query = "type = card AND familiarity < 0.4 ORDER BY familiarity"
```

## Safe and fast

A query only reads item fields; it can never change anything. A query may be up to 2000 characters long. Even with
10 000 items the result comes back in a few milliseconds.

See also the [[library]] and the [[faq]].
