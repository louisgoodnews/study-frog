# Scripting in StudyFrog: Python over the socket, an embedded DSL, or FrogScript

Status: research and recommendation for STORY-0150 (2026-11-22). Decides nothing that is already accepted;
it does not change ADR 0013.

## The question
Should StudyFrog embed a lightweight scripting language (Rhai, Rune, Lua, or a study-query DSL) so users can
script without a Python runtime, or keep the current approach (Python over the control protocol, ADR 0013)?

## What already exists (from the code, not from memory)
Three routes are in the tree today, so the real question is whether a fourth is needed.

1. **Python out of process** (ADR 0013, accepted, M12). Scripts are child processes that speak the control
   protocol over a Unix socket or stdio. The `studyfrog` Python client is standard library only. Runs under
   a timeout, with an access level (`read`, `write`, `destructive`) per script. Cost: a process start of
   about 30 ms per run, and `python3` must exist on the machine. Unix sockets only; the stdio transport
   works everywhere.
2. **FrogScript, an in-tree DSL** (`crates/studyfrog-script`, about 3,300 lines, pure Rust, no dependencies
   beyond serde and thiserror). Its spec (`docs/specs/frogscript.md`, section 7.2) sets a step cap of 100,000
   operations per run. The step cap is implemented: `consume_fuel` in `crates/studyfrog-script/src/eval_expr.rs`
   lines 18 to 23, and the loop handling in `eval.rs`. It runs in-app, with no process and no network.
   It has vault objects (stacks, cards, tags) and logging that goes to ScriptStudio's streams.
3. **The control protocol itself** (`studyfrog --control-stdio`, `studyfrog ctl`), the API both 1 and the
   console use. Any language that can write JSON to a socket can script the app today.

Two gaps in the existing material, to fix whichever way the decision goes:
- The FrogScript spec promises a 16 MB memory ceiling per script (section 7.2). I could not find it in the
  code: `grep` for a byte limit in `crates/studyfrog-script/src` found nothing. The step cap is real; the
  memory cap is documentation only until implemented or removed from the spec.
- The spec's section 7.1 says FrogScript lives in `crates/studyfrog-service` and `crates/studyfrog-ui`.
  The code is in `crates/studyfrog-script`. The spec should be corrected.

## The candidates, compared
Criteria are the ones the story names: security isolation, memory safety, cross-platform ergonomics, and
latency. Ergonomics also counts the cost of a second language for users and for us.

| | Python over socket (ADR 0013) | Embedded Rhai or Rune | Lua via mlua | FrogScript (in tree) |
|---|---|---|---|---|
| Runs in-app with no process | no (about 30 ms start) | yes | yes | yes |
| Needs an install on the user's machine | `python3` | nothing | nothing | nothing |
| Memory safety of the host | separate process; a crash cannot touch the app | pure Rust | C library behind FFI (inside a dependency) | pure Rust |
| Sandbox | process rights plus the control policy | only what the host registers; operation and depth limits exist in Rhai (from memory, to verify) | host registers functions; a memory limit is available | step cap in place; memory cap not implemented |
| Fits `unsafe_code = forbid` in our crates | yes | yes | our code yes; C library in a dependency | yes |
| Users already know it | high | low | medium | low |
| Ecosystem and libraries | large | small | large | none |
| Cost to StudyFrog now | none (shipped) | new crate, bindings, docs, a second DSL to teach and test | new C dependency and packaging | none (shipped) |
| Timeouts and kill | yes (process) | by step limit | by hook | by step cap (fuel) |

Notes on the table:
- The Rhai and Lua memory and operation limits are from general knowledge of those projects, not measured
  here. Confirm before relying on them.
- "Pure Rust" for Rune is my understanding; check its current state before any decision.

## Reasoning
- **The main benefit of an embedded engine is the in-app latency and no install.** FrogScript already gives
  both, for the study-automation use it was designed for. The console's "instant evaluation" can use it.
- **A second embedded DSL would split the scripting story** into three languages (Python, FrogScript, Rhai).
  Each needs docs, tests, editor support and a stable API. That cost is larger than the gain.
- **Python out of process is the safest route for arbitrary user code**, and ADR 0013 already chose it for
  that reason. Its costs (the install, the 30 ms start) matter less for scheduled jobs and exports than for
  interactive use, which FrogScript covers.
- **Lua would bring a C dependency** into a project that forbids `unsafe` in its own code and prizes a
  simple static build. Not worth it.

## Recommendation
1. **Do not embed Rhai, Rune or Lua now.** Keep Python over the control protocol for user scripts (ADR 0013),
   and keep FrogScript as the in-app DSL.
2. **Make FrogScript the console's language for quick, interactive evaluation**, with live highlighting and
   autocompletion of its keywords and vault objects (the editor part of STORY-0150). This is where the
   "instant" requirement is met.
3. **Close the gaps first:** implement or remove the 16 MB memory cap, and correct the spec's crate paths.
4. **Reconsider only if** a user need appears that FrogScript cannot express and Python cannot serve in
   time (for example, a per-keystroke evaluation in the editor). That would be a new ADR, with a measured
   latency and a memory test, not an assumption.

## What this research did not do
- No benchmark of FrogScript against a Rhai or Lua build was run; the latency comparison is by design, not
  measured. The Python start time is the one in ADR 0013 (about 30 ms), measured by its authors.
- No check of Rune's current state or of Rhai's exact limit API.
