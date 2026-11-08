# Inline cards in notes (STORY-0110)

A note can hold cards and questions inline. The parser is `studyfrog_md::inline_extract` (pure); the
sync is `studyfrog_service::inline_sync`; the window shows it on the note's page.

## Syntax
- `Term :: Definition` on its own line, also as a list item (`- `, `* `, `+ `): a card, front `Term`,
  back `Definition`. Both sides must be non-empty; the separator is ` :: ` with spaces.
- `Q: prompt` followed (later, before the next prompt) by `A: answer`: a card.
- `??: prompt` followed by `A: answer`: a question with a reference answer.
- Lines inside fenced code (```` ``` ```` or `~~~`) are ignored. Each block remembers the nearest heading
  above it. The note file itself is never changed: the syntax is plain text in the body, so the Markdown
  round trip keeps it as written.

## Sync
"Extract / Sync N card(s)…" appears in the toolbar of a note's page when the body (as being edited)
has inline blocks. It opens a panel listing every block as **new**, **updated** or **unchanged** and
the stack folder; "Sync cards" carries it out, "Undo the sync" (and the Library's Undo) takes it back.

- Items go into the note's own stack folder; a note outside a stack is refused.
- Every extracted item carries `extracted_from: <note ULID>` and `extract_key: <16 hex digits>` (FNV-1a
  of the kind and the front, lower-cased with spacing evened out) as front-matter keys. A sync finds
  the items of the folder with the note's id and matches them by key, so:
  - a block without an item is created (title from the front, tags of the note, the usual priority,
    difficulty and item key);
  - a block whose item has another answer gets the answer replaced; the file, id and schedule stay;
  - a block whose item is current is left alone; a front written twice in the note counts once.
- The answer of an extracted item ends with `Source: [[Note title#Heading]]` (or `[[Note title]]`
  outside a heading). The link makes the note list the item among its backlinks; answer comparison
  ignores this last line.
- Changing a block's front makes a new item (the key follows the front); the old item stays and keeps
  its schedule. The note is the source of the answer: editing an extracted item's answer by hand is
  overwritten by the next sync that sees a different definition.
- The MCP tool `note_extract_inline_cards` (STORY-0119) uses the same parser.
