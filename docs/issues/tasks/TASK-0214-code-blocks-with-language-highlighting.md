---
id: TASK-0214
type: task
title: Code blocks with language highlighting
status: done
epic: EPIC-026
parents: [STORY-0074]
estimate: 5
labels: [m11, ui, md]
acceptance_criteria:
  - "A fenced code block with a language (```python) is drawn with its keywords, strings, comments and numbers in colours that work in the dark and the light theme and carry no meaning by colour alone"
  - "The languages are at least: python, r, rust, php, java, toml, html, json, csv, tsv, cpp (the owner: 'etc.', so the list is a table that grows)"
  - "An unknown or missing language is drawn plain"
  - "Highlighting is computed when the text changes, not every frame, and a 5 000 line block stays within the frame budget"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Code in notes and cards should be readable.

## Acceptance Criteria
- [x] A fenced code block with a language (```python) is drawn with its keywords, strings, comments and numbers in colours that work in the dark and the light theme and carry no meaning by colour alone
- [x] The languages are at least: python, r, rust, php, java, toml, html, json, csv, tsv, cpp (the owner: 'etc.', so the list is a table that grows)
- [x] An unknown or missing language is drawn plain
- [x] Highlighting is computed when the text changes, not every frame, and a 5 000 line block stays within the frame budget

## Design Notes
Done as `highlight.rs` and `highlight_langs.rs` in `studyfrog-md` (built, not `syntect`: see the comparison at the top of `highlight.rs`), tokens stored in `Block::Code` when the text is parsed, drawn by `preview_rich::code`; colours from `theme_hue.rs`. python, r, rust, php, java, cpp/c, toml, json, html/xml, csv and tsv; a 5 000 line block tokenizes in a few milliseconds.

Build or buy (P10): compare `syntect` (many grammars, large, regex engine) with a small in-house highlighter driven by a table per language (keywords, comment markers, string quotes, number form); record the comparison in the task. For eleven languages and the five token kinds above an in-house table-driven tokenizer is likely smaller and faster; JSON, TOML, CSV and TSV need only a few rules. Cache the tokens next to the parsed blocks as the preview does.

## Dependencies
TASK-0197 for the block drawing.

## Open Questions
None.
