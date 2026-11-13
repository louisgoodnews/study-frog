---
id: EPIC-045
type: epic
title: "Macro library expansion and escaping syntax"
status: done
parents: []
labels: [m12, markdown, ui]
acceptance_criteria:
  - "The markdown macro engine supports a rich catalog of semantic content macros including rules, headers, directional arrows, callout panels, hidden answers, and inline formatting"
  - "An escaping syntax ({#macro}) prevents accidental expansion when typing literal macro names"
  - "The macro system provides an architecture ready for custom user-authored Python macro plugins"
---

## Context
StudyFrog's text macro engine allows instant expansion of complex markdown snippets (such as tables, code blocks, or colored text) upon typing the closing brace `}`. Learners have requested a much broader palette of macros (e.g. `{rule}`, `{header:level}`, `{arrow_right}`, `{arrow_left}`, `{hide}`, `{panel}`, `{link}`) along with an escape syntax (`{#code}`) to prevent expansion when discussing or documenting macros literally.

## Description
Deliver an expanded markdown macro system:
1. **Macro Library Expansion (`STORY-0143`):** Implement a comprehensive macro catalog in `crates/studyfrog-ui/src/md_macros.rs` covering semantic layout, navigation arrows, spoilers/hiding, and formatting shortcuts.
2. **Escaped Macro Syntax (`STORY-0144`):** Support leading `#` in macro triggers (e.g. `{#code}`, `{#table}`) so users can type literal macro syntax without triggering unwanted snippet expansion.

## Acceptance Criteria
- [x] The markdown macro engine supports a rich catalog of semantic content macros including rules, headers, directional arrows, callout panels, hidden answers, and inline formatting
- [x] An escaping syntax ({#macro}) prevents accidental expansion when typing literal macro names
- [x] The macro system provides an architecture ready for custom user-authored Python macro plugins

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/md_macros.rs`, `crates/studyfrog-ui/src/view/md_editor/toolbar.rs`.
- Child stories: STORY-0143, STORY-0144.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- None.

## Open Questions
- None.
