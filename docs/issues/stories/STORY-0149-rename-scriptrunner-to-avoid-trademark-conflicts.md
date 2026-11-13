---
id: STORY-0149
type: story
title: "Rename ScriptRunner across application and documentation to avoid trademark conflicts"
status: done
epic: EPIC-049
parents: [EPIC-049]
estimate: 2
labels: [m12, ext, console]
acceptance_criteria:
  - "All occurrences of 'ScriptRunner' in UI labels, headers, buttons, and tooltips are renamed to a trademark-safe brand name"
  - "Documentation, research papers, specifications, and help files are updated to use the new naming convention"
  - "Internal identifier tokens, element IDs, and CSS/layout keys are migrated without breaking user configuration"
---

## Context
Adaptavist holds registered trademarks on "ScriptRunner" in the software automation and development ecosystem. To eliminate trademark infringement risks and establish an authentic brand identity for StudyFrog's built-in developer tooling, all references to "ScriptRunner" must be systematically replaced with a unique, trademark-safe name (such as **FrogScript** or **ScriptStudio**).

## Description
Perform a complete naming migration across UI, codebase, and documentation:
1. Select a proprietary name (recommended: **FrogScript** or **ScriptStudio**) that aligns with StudyFrog's persona and technical architecture.
2. Update UI component labels, buttons (`"Run FrogScript"`, `"Hide FrogScript"`), headers, and burger menu entries.
3. Migrate internal IDs and element salts (`scriptrunner-code-buffer`, `scriptrunner-multiline-text`) gracefully.
4. Update markdown documentation, release notes, research papers, and specifications.

## Acceptance Criteria
- [x] All occurrences of 'ScriptRunner' in UI labels, headers, buttons, and tooltips are renamed to a trademark-safe brand name
- [x] Documentation, research papers, specifications, and help files are updated to use the new naming convention
- [x] Internal identifier tokens, element IDs, and CSS/layout keys are migrated without breaking user configuration

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `crates/studyfrog-ui/src/view/burger_menu.rs`, `docs/specs/scripting.md`, `docs/research/`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-049 (Expanded scripting environment, DSL, job scheduling, and execution logging).

## Open Questions
- Final name preference: "FrogScript Studio" vs. "ScriptStudio" vs. "FrogScript".
