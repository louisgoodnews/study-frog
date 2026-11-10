---
id: STORY-0143
type: story
title: Comprehensive markdown macro library expansion
status: done
epic: EPIC-045
parents: [EPIC-045]
estimate: 3
labels: [m12, markdown, ui]
acceptance_criteria:
  - "The macro library expands new macros: {rule}, {header:level}, {arrow_right}, {arrow_left}, {hide}, {panel}, {link}, {bold:text}, and {italic:text}"
  - "The editor macro dropdown menu lists all available macros with usage hints and descriptions"
  - "Macro expansions place cursor selection at the appropriate editable parameter"
---

## Context
Writing notes and study cards with rich markdown formatting should be fluid and keyboard-driven. Adding specialized macros for rules, headers, arrows, callout panels, and inline styling eliminates repetitive typing and memorizing arcane syntax.

## Description
Extend `MACROS` in `crates/studyfrog-ui/src/md_macros.rs`:
- `{rule}`: Expands to a markdown divider `---`.
- `{header:level}` or `{h:N}`: Expands to `# Header` with $N$ hash marks.
- `{arrow_right}` or `{->}`: Expands to `->` or Unicode arrow `->`.
- `{arrow_left}` or `{<-}`: Expands to `<-`.
- `{hide}` or `{spoiler}`: Expands to `<details><summary>Reveal</summary>Hidden answer</details>`.
- `{panel}` or `{callout}`: Expands to `> [!NOTE]\n> Note content`.
- `{link}`: Expands to `[Link text](url)`.
- `{bold:text}` or `{b:text}`: Expands to `**text**`.
- `{italic:text}` or `{i:text}`: Expands to `*text*`.
- `{graph}`: Expands to a mermaid xy-chart or bar template.

## Acceptance Criteria
- [x] The macro library expands new macros: {rule}, {header:level}, {arrow_right}, {arrow_left}, {hide}, {panel}, {link}, {bold:text}, and {italic:text}
- [x] The editor macro dropdown menu lists all available macros with usage hints and descriptions
- [x] Macro expansions place cursor selection at the appropriate editable parameter

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/md_macros.rs`, `crates/studyfrog-ui/tests/md_macros.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-045 (Macro library expansion and escaping syntax).

## Open Questions
- None.
