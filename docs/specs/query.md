# Query language

The typed search of the library (STORY-0057), in the manner of Jira's JQL and written like SQL. Planned for 0.3.0; the syntax below is
a proposal to be accepted with TASK-0129.

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

`!~` and `NOT` negate; `field != value` is the same as `NOT field = value`. A comparison with `IS EMPTY` on a field that is never empty is an error that
says so.

## Errors
A query that cannot be read is not run. The message names the **column** (counted from 1) and what was expected, and the box marks the place:
| Query | Message |
|---|---|
| `type =` | Expected a value after `=` at column 6 |
| `status = nope` | Unknown status `nope` at column 10. The statuses are new, learning, reviewing, done, paused |
| `colour = red` | Unknown field `colour` at column 1. Did you mean `tag`? (names within two edits are offered) |
| `due < 7d AND` | Expected a condition after `AND` at column 14 |
| `(type = card` | Missing `)` for the `(` at column 1 |
| `text ~ "open` | The string that starts at column 8 is not closed |
| `familiarity > 1.5` | `familiarity` is between 0 and 1 (column 15) |
| `type > card` | `>` does not apply to `type`; use `=`, `!=` or `IN` (column 6) |

## Safety and limits
The query is parsed into a tree and the tree is evaluated against the index with **bound parameters**: user text is never put into SQL, nothing but
item fields can be reached, and there are no sub-queries or functions. A query is at most 2000 characters and 20 levels of parentheses deep.
`familiarity` is not stored: it is computed for the items that pass the other conditions first.

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
