# Markdown schema

Status: draft (see [ADR 0005](../adr/0005-markdown-schema-and-frontmatter.md)).

## Layout
```
vault/
  <stack-folder>/
    _stack.md        # kind: stack
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

## Per kind
**stack** (`_stack.md`): `description` (string), `srs` (override map, optional),
`daily_new_limit` (int), `daily_review_limit` (int).

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
