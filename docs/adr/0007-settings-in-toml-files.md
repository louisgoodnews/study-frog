# ADR 0007: Settings in TOML files, content in Markdown

- Status: **Proposed** (implemented provisionally; owner review requested)
- Date: 2026-10-30
- Amends: [ADR 0005](0005-markdown-schema-and-frontmatter.md) and [ADR 0001](0001-workspace-and-crate-layout.md)

## Context
Stack settings (scheduler, retention, daily limits) were first modelled as `_stack.md` front-matter. The owner
asked whether configuration would be better in JSON, RON or TOML files that sit beside the Markdown. Reasons it is:
- Settings are not content. Front-matter keys the app invented pollute notes that Obsidian-style tools read.
- Rewriting YAML front-matter drops comments on known keys (ADR 0005); settings files are exactly where users
  write comments.
- There was no home at all for vault-wide settings (default scheduler, FSRS weights, interval cap).
- Settings change independently of notes; editing a config should not touch a Markdown file.

## Decision
- **Split by nature.** Front-matter keeps what belongs to one item and must travel with its file: `id`, `kind`,
  `title`, `tags`, `aliases`, timestamps, per-kind item keys. Everything that configures behaviour moves to TOML.
- **Two files.** `studyfrog.toml` at the vault root (vault defaults) and `_stack.toml` next to `_stack.md`
  (stack settings). A stack is still a folder with a `_stack.md`; `_stack.md` keeps the stack's `id`, `title`,
  `description` and body. Config is keyed by folder (and by id where needed), never by a list of Markdown paths,
  so renames stay safe.
- **TOML**, not RON or JSON: comments, easy to edit by hand, no indentation pitfalls; RON is Rust-specific and
  JSON has no comments. Parsing uses the `toml` crate (a hard general problem, P10); rendering is hand-written.
- **New crate `studyfrog-config`** (pure: types come from `core`, parse and render, validation with line numbers).
  Allowed dependencies: `core`. `store`, `service` and `ui` may depend on it.
- Unknown keys are errors; invalid files fall back to defaults and are reported.

## Consequences
+ Notes stay clean; settings keep their comments; one obvious place for every kind of setting.
+ The index and service logic are unchanged by where settings come from (`stack_config` mirrors `_stack.toml`).
- Two files per configured stack (`_stack.md` + `_stack.toml`) instead of one.
- Rendering with the `toml` serializer would lose comments on edit; the settings UI should use `toml_edit` when it
  arrives. The renderer here only creates files.
- Old `_stack.md` files that carry `daily_*` or `srs` keys keep them as inert unknown keys.

## Alternatives
- Everything in front-matter (the previous state): rejected above.
- An explicit manifest listing Markdown paths: duplicates the folder convention and breaks on rename.
- One global settings file only: stack-level settings would drift from the stack they describe.
