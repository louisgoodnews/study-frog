---
id: STORY-0150
type: story
title: "In-app code execution console and domain-specific language (DSL) research"
status: backlog
epic: EPIC-049
parents: [EPIC-049]
estimate: 5
labels: [m12, ext, console, research]
acceptance_criteria:
  - "The in-app scripting console provides an interactive code execution environment with instant evaluation of user scripts"
  - "Research document evaluates the feasibility, trade-offs, ergonomics, and security boundaries of embedding a lightweight scripting DSL versus standard Python"
  - "The console supports interactive command and script execution with error highlighting and contextual autocompletion"
---

## Context
Learners and power users want to interact directly with their flashcards, decks, and review telemetry through code without leaving the StudyFrog desktop window. While external Python scripting over Unix domain sockets is supported, users desire an integrated, zero-latency in-app scripting console. Additionally, research is required to evaluate whether a lightweight embedded Domain-Specific Language (DSL)—such as Rhai or a sandboxed study-query DSL—should be embedded directly into StudyFrog to allow scripting without requiring an external Python runtime.

## Description
Expand console capabilities and author research on DSL integration:
1. Conduct research and author `docs/research/scripting-dsl-evaluation.md` comparing:
   - Python over local socket (current ADR 0013 approach)
   - Embedded Rust-native DSL (e.g. Rhai, Rune, or custom query DSL) for instant, sandboxed execution without external dependencies
   - Security isolation, sandbox permissions, memory safety, and cross-platform ergonomics
2. Enhance the in-app code editor with live syntax highlighting, bracket matching, and interactive execution feedback.
3. Provide an interactive evaluation engine allowing code execution directly within the desktop UI.

## Acceptance Criteria
- [ ] The in-app scripting console provides an interactive code execution environment with instant evaluation of user scripts
- [ ] Research document evaluates the feasibility, trade-offs, ergonomics, and security boundaries of embedding a lightweight scripting DSL versus standard Python
- [ ] The console supports interactive command and script execution with error highlighting and contextual autocompletion

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs`, `docs/research/`, `crates/studyfrog-service/src/scripting/`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-049 (Expanded scripting environment, DSL, job scheduling, and execution logging).
- STORY-0149 (Rename ScriptRunner to avoid trademark conflicts).

## Open Questions
- Whether a custom DSL should compile down to StudyFrog control protocol JSON-RPC commands or run as native in-process AST evaluations.
