---
id: STORY-0225
type: story
title: "Split files over 250 lines and fix the rehearsal code hand-merged from the choice branch"
status: done
epic: EPIC-072
parents: ['EPIC-072']
estimate: 3
labels: [m14, ui, rehearsal, tooling]
acceptance_criteria:
  - "crates/studyfrog-ui/src/exec_run.rs (263 lines) and crates/studyfrog-ui/src/view/run.rs (252 lines) are each at most 250 lines, by moving code into new modules with the same behaviour"
  - "crates/studyfrog-ui/src/control_plan/stacks.rs has no unused imports"
  - "The rehearsal choice code merged by hand (STORY-0222) uses main's MsgRehearsal and Effect layout, with no leftover branch-only names"
  - "scripts/linecount.sh 250 passes for the whole tree"
---

## Context
After the merge run, two files are over the 250-line limit and one control module has unused imports. The rehearsal
choice code (STORY-0222) was adapted by hand to the current message layout and has not been checked by a compiler.

## Description
Move code out of the two large files into well-named modules, remove the unused imports, and check the hand-merged
names against main's layout.

## Acceptance Criteria
- [x] exec_run.rs and view/run.rs are at most 250 lines each (103 and 154 lines)
- [x] No unused imports in control_plan/stacks.rs (`cargo check -p studyfrog-ui` reports no warnings)
- [x] Hand-merged rehearsal choice code uses main's names (it compiles against main's layout; no branch-only names remain)
- [x] linecount passes (the gate's line-count step reports no file over 250 lines)

## Design Notes
Behaviour must not change. Keep public function names and the module paths the rest of the code uses, or update their
callers in the same change.

## Dependencies
- The merge of STORY-0223/0224 (branch worktree-agent-ac7e0f138a7daca29) should land first, since it touches the same
  rehearsal code paths.

## Open Questions
- None.
