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

## Implementation notes (M2)
- **Key order on rewrite:** known keys are written in canonical order (common keys, then the
  kind's own keys), defaults omitted. Unknown keys, and keys that belong to another kind, are kept
  verbatim after them in their original relative order. Comments on known keys are not preserved.
- **Single-line fields** (`title`, `tags`, `aliases`, `description`... entries) must be non-blank and
  contain no control or Unicode line-break characters.
- **`srs:`** accepts only `request_retention` for now; any other override key is an error.
- **Timestamps:** RFC 3339, years 0000-9999; written as UTC with milliseconds.
- **`???` separator:** first line that is exactly `???` outside a fenced code block (``` or ~~~);
  a card or question without one is an error. Wikilinks may combine forms:
  `[[Target#Heading|label]]`; empty targets are ignored. Links in code spans and fences are ignored.
- **Stack body:** `_stack.md` may have a Markdown body; it is preserved.
