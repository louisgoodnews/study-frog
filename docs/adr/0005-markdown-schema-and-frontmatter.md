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

## YAML crate decision (resolved 2026-10-30)
**`serde-saphyr` 2.0.0** (pure Rust, no libyaml/unsafe, MIT OR Apache-2.0, MSRV 1.89) for typed
deserialisation and serialisation of front-matter.
- Validated: `tools/repo-lint` parses all issue front-matter with it, including
  `deny_unknown_fields` and error positions.
- Rejected: `serde_yaml` (archived 2024), `serde_yml` (archived after unsoundness reports),
  `serde_yaml_ng` (RUSTSEC lists its libyaml backend as unmaintained). `serde_norway` and
  `yaml_serde` are drop-in forks; not evaluated further because `serde-saphyr` avoids the
  libyaml lineage altogether.
- The crate stays isolated inside `studyfrog-md`, so a later swap is local.

## Amendment (Proposed, needs owner approval): drop `gray_matter`
`gray_matter` 0.3.2 was last released 2025-07-10, parses via an optional `yaml-rust2` into a
generic value, and cannot write front-matter. The round-trip requirement (`parse(render(x)) == x`)
needs serialisation anyway, and splitting on the `---` fences is about ten lines
(`split_front_matter` in `tools/repo-lint/src/issues.rs` is the prototype). Proposal: own the
fence splitter in `studyfrog-md` and use `serde-saphyr` in both directions, removing one
dependency and one second YAML parser. Until approved, TASK-0021 keeps its title unchanged.

## Risks
- `comrak` is the alternative to `pulldown-cmark` if GFM extensions prove insufficient.
- `serde-saphyr` is younger than the libyaml-based crates; mitigated by golden and round-trip tests.
