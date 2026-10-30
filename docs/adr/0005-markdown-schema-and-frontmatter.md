# ADR 0005: Markdown schema and front-matter

- Status: **Accepted** (2026-10-30)
- Date: 2026-10-30

## Decision
- One item per file (`.md`); a stack is a folder with a `_stack.md` definition file.
- Front-matter is YAML delimited by `---`, parsed by `gray_matter` into typed structs
  with `serde`. Keys are specified in [markdown-schema](../specs/markdown-schema.md).
- Required keys on every item: `id` (ULID), `kind` (`note|card|question`), `title`.
- Body is CommonMark parsed by `pulldown-cmark`; `[[wikilinks]]` are extracted by a small
  scanner over text events. A card or question body splits front/back on a line `???`.
- Unknown front-matter keys are preserved on round-trip (stored as a remainder map).
- The parser never rewrites a file unless the app changes it; rewrites preserve key order.

## Risks
- `serde_yaml` is archived/unmaintained. Decide at M2 between `serde_yaml`, a maintained fork,
  or the YAML backend `gray_matter` uses. The choice is isolated inside `studyfrog-md`.
- `comrak` is the alternative to `pulldown-cmark` if GFM extensions prove insufficient.
