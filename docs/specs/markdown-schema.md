# Markdown schema

Status: draft (see [ADR 0005](../adr/0005-markdown-schema-and-frontmatter.md)).

## Layout
```
vault/
  <stack-folder>/
    _stack.md        # kind: stack (identity, description, body)
    _stack.toml      # optional settings of the stack, see config-files.md
    <anything>.md    # kind: note | card | question
```

## Common keys (all kinds)
| Key | Type | Req | Notes |
|---|---|---|---|
| `id` | ULID string | yes | stable identity; never reused |
| `kind` | `stack\|note\|card\|question` | yes | |
| `title` | string | yes | rehearsable for notes |
| `tags` | list of string | no | |
| `aliases` | list of string | no | extra link targets |
| `created`, `updated` | RFC 3339 datetime | no | set by app |

## Stacks, folders and references
(Decided in [ADR 0011](../adr/0011-stack-folders-and-references.md).) A folder is a **stack** when it holds `_stack.md` and
a **normal folder** otherwise. An item's **primary stack** is the nearest stack folder above its file, none meaning the
Inbox. The optional key `referenced_by` on a note, card or question lists the *other* stacks that reference it, by stack
id (a ULID): `referenced_by: [01HZ..., 01HY...]`. It never lists the primary stack or a stack twice, and a reference to a
stack that does not exist is a vault problem. The file holds nothing else about stacks: the memory, schedule, due date,
familiarity and difficulty of an item in each stack live in the database, never in the file.

## Per kind
**stack** (`_stack.md`): `description` (string). Stack *settings* (scheduler, retention, daily limits)
are not front-matter: they live in `_stack.toml` ([config-files](config-files.md)). Old files that still carry
`daily_new_limit`, `daily_review_limit` or `srs` keep them as verbatim unknown keys; they are ignored.

**note**: no extra keys. Body is free Markdown with `[[wikilinks]]`.
`rehearse_title: true|false` (default false): title is rehearsed against the body summary.

**card**: `reversible` (bool, default false). Body: front text, a line containing only `???`,
then back text.
```markdown
---
id: 01JABCDEF0123456789ABCDEFG
kind: card
title: Capital of France
---
What is the capital of France?
???
Paris
```

**question**: `accepted` (list of string, optional alternative reference answers),
`rubric` (list of required key points, optional), `min_score` (float 0..1, default 0.7).
Body: prompt, `???`, reference answer.

## Rules
- Unknown keys are preserved on round-trip.
- Missing/invalid required keys produce a typed parse error carrying file and line; the
  scanner skips the file and reports it rather than aborting.
- Duplicate `id` across files is an error reported on both files.
- Wikilink forms: `[[Title]]`, `[[Title|label]]`, `[[Title#Heading]]`; resolved by title or alias.

## Implementation notes (M2)
- **Key order on rewrite:** known keys are written in canonical order (common keys, then the
  kind's own keys), defaults omitted. Unknown keys, and keys that belong to another kind, are kept
  verbatim after them in their original relative order. Comments on known keys are not preserved.
- **Single-line fields** (`title`, `tags`, `aliases`, `description`... entries) must be non-blank and
  contain no control or Unicode line-break characters.
- **Settings are not front-matter** (decided in [ADR 0007](../adr/0007-settings-in-toml-files.md)): front-matter
  holds identity and per-item metadata only, so notes stay clean for other Markdown tools.
- **Timestamps:** RFC 3339, years 0000-9999; written as UTC with milliseconds.
- **`???` separator:** first line that is exactly `???` outside a fenced code block (``` or ~~~);
  a card or question without one is an error. Wikilinks may combine forms:
  `[[Target#Heading|label]]`; empty targets are ignored. Links in code spans and fences are ignored.
- **Stack body:** `_stack.md` may have a Markdown body; it is preserved.

## Pictures
`![description](assets/name.png)` shows a picture. The path is relative to the vault root and must stay inside it
(no `..`, no absolute path, no web address); only png, jpg and gif are shown. The app copies pictures into
`assets/` and names them after the file and a short hash of its content.

## Front-matter keys the app writes
Besides `id`, `kind`, `title`, `tags`, `aliases`, `created` and `updated`: `priority` (`lowest` to `highest`),
`difficulty` (`easy`, `medium`, `difficult`), `status` (planned: `new`, `learning`, `reviewing`, `done`, `paused`; see [status](status.md)) and, on a stack, `subjects` (comma-separated, or a flow sequence of double-quoted names when a name holds a comma, a quote, a backslash, a bracket or blanks at its ends: `subjects: ["Statistics, advanced", Maths]`; see `studyfrog_md::names`). Unknown keys are kept. `referenced_by` (a flow sequence of stack ids) is written only when an item has references; see above.
