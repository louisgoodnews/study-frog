# Query language

The typed search of the library (STORY-0057), in the manner of Jira's JQL and written like SQL. Planned for 0.3.0; the syntax below is
accepted by the owner on 2026-11-02.

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

## Grammar
```
query      = [ or ] [ "ORDER" "BY" order { "," order } ]
or         = and { "OR" and }
and        = not { "AND" not }
not        = [ "NOT" ] primary
primary    = "(" or ")" | comparison
comparison = field cmp value
           | field [ "NOT" ] "IN" "(" value { "," value } ")"
           | field "IS" [ "NOT" ] "EMPTY"
           | field ( "~" | "!~" ) string
cmp        = "=" | "!=" | "<" | "<=" | ">" | ">="
order      = field [ "ASC" | "DESC" ]
value      = word | string | number | percent | date | relative
```
- Keywords (`AND`, `OR`, `NOT`, `IN`, `IS`, `EMPTY`, `ORDER`, `BY`, `ASC`, `DESC`) and field names are not case-sensitive.
- A **word** is letters, digits, `-`, `_`, `.` and `/`; anything else (spaces, commas, parentheses) needs a **string** in double quotes, where `\"` and `\\` escape.
- A **number** is a decimal number; a **percent** is a number followed by `%` (`40%` is 0.4). A **date** is `2026-11-18` or `today`. A **relative**
  date is a signed count and a unit: `7d` (seven days from today), `-14d` (fourteen days ago), `2w`, `1m`, `1y`; `today` is `0d`.
- An empty query matches every item. `ORDER BY` takes at most three keys; items equal on all keys are sorted by path.

## Fields
| Field | Type | Operators | Notes |
|---|---|---|---|
| `text` | text | `~` `!~` | title and body together, case-insensitive |
| `title` | text | `=` `!=` `~` `!~` | `=` is the whole title, case-insensitive |
| `type` | word | `=` `!=` `IN` | `note`, `card`, `question` |
| `status` | word | `=` `!=` `IN` | the effective status: `new`, `learning`, `reviewing`, `done`, `paused` ([status](status.md)) |
| `stack` | word or string | `=` `!=` `IN` | a stack's folder or title; includes the stacks below it |
| `subject` | word or string | `=` `!=` `IN` `IS EMPTY` | the subjects of the item's stack, inherited |
| `teacher` | word or string | `=` `!=` `IN` | the teachers of those subjects |
| `tag` | word or string | `=` `!=` `IN` `IS EMPTY` | true when the item has the tag |
| `priority` | ordered word | `=` `!=` `<` `<=` `>` `>=` `IN` | `lowest` < `low` < `medium` < `high` < `highest`; the effective value, own or inherited |
| `difficulty` | ordered word | the same, and `IS EMPTY` | `easy` < `medium` < `difficult`; the effective value, if any |
| `familiarity` | number 0 to 1 | `=` `!=` `<` `<=` `>` `>=` | a percent is accepted; items with no history have none (use `IS EMPTY`) ([familiarity](familiarity.md)) |
| `due` | date | `=` `!=` `<` `<=` `>` `>=` `IS EMPTY` | the next review; empty for an item never rehearsed |
| `reviews` | whole number | the number operators | how many reviews the item has |
| `created`, `updated` | date | the date operators | from the front-matter |
| `path` | text | `=` `~` | the file's path in the vault |
| `key` | text | `=` `!=` `~` `!~` `IN` `IS EMPTY` | the item's key (`FLASHCARD_12`), case-insensitive, `NOTE_012` is `NOTE_12`; sorts by kind, then number. In the Standard search a word that is a key finds that item |

`!~` and `NOT` negate; `field != value` is the same as `NOT field = value`. A comparison with `IS EMPTY` on a field that is never empty is an error that
says so.

## Errors
A query that cannot be read is not run. The message names the **column** (counted from 1) where something is wrong or missing, and the box marks the place.
For something missing the column is the one right after the last token, where it should have been:
| Query | Column | Message |
|---|---|---|
| `type =` | 7 | Expected a value after `=` at column 7 |
| `status = nope` | 10 | Unknown status `nope` at column 10. The statuses are new, learning, reviewing, done, paused |
| `colour = red` | 1 | Unknown field `colour` at column 1. The fields are text, title, type, ... (a name within two edits is offered instead: `tga` gives Did you mean `tag`?) |
| `due < 7d AND` | 13 | Expected a condition after AND at column 13 |
| `(type = card` | 1 | Missing `)` for the `(` at column 1 |
| `text ~ "open` | 8 | The string that starts at column 8 is not closed |
| `familiarity > 1.5` | 15 | familiarity is between 0 and 1 (column 15) |
| `type > card` | 6 | `>` does not apply to `type`; use `=`, `!=` or `IN` (column 6) |
| `priority IS EMPTY` | 10 | `priority` is never empty (column 10) |
| `type = card status` | 13 | Unexpected `status` at column 13. Expected AND, OR or ORDER BY |
| `type IN card` | 9 | Expected `(` after IN at column 9 |
| `ORDER BY text` | 10 | A result cannot be sorted by `text` (column 10) |

A field that can be missing (difficulty, familiarity, due, created, updated) matches no comparison except `!=` when it is missing; use `IS EMPTY` to find those items.

## Safety and limits
The query is parsed into a tree of typed values (a name, an operator and a value already checked against its field) and the tree is evaluated **in memory** over the
items' rows, which are read from the index once: user text is never put into SQL, nothing but item fields can be reached, and there are no sub-queries or functions. A query is at most 2000 characters and 20 levels of parentheses deep.
`familiarity` is not stored in the index: it is computed when the rows are loaded. Measured: 10 000 items are filtered and sorted in 2 to 18 ms (debug build; budget 50 ms).

## Standard mode
The Standard mode's fields compose into a query in this language, and the query box shows it; typing in the query box fills the Standard fields
when it can (only `AND` of simple comparisons can), and says "Query only" when it cannot.

## Completion
Field names, operators and, after a field, its values (statuses, types, stacks, subjects, tags) are offered as the user types; after `ORDER BY` the
fields.

## Saved filters
`filters.toml` in the vault:
```toml
[[filter]]
name = "Hard cards"
query = "type = card AND familiarity < 0.4 ORDER BY familiarity"
```

## Tests
A corpus of valid queries parses and prints back to an equivalent query; every row of the error table gives that message at that column;
a property test feeds random text and never panics; a test shows that a hostile string (quotes, `;`, `--`, `%`) is only ever a value.
