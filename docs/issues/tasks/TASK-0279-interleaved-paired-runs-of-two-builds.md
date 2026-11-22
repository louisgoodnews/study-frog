---
id: TASK-0279
type: task
title: "Interleaved paired runs of two builds (baseline and new, alternating)"
status: backlog
epic: EPIC-067
parents: ['STORY-0201']
estimate: 2
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'bench-reference.sh alternates invocations of two built bench binaries (A, B, A, B, ...) for one case, so each pair shares the machine state'
  - 'Each side is written to its own .ns file in the same format, so scripts/bench-compare.sh compares the pair'
---

## Context
EPIC-067 Decisions: 5 interleaved invocations per side, baseline and new alternating, so the comparison is paired.
TASK-0278 repeats one build only, so it does not yet meet that decision.

## Description
Interleaving needs the two binaries. Plan: `cargo bench --no-run` reports each bench executable's path; the
owner copies the baseline binary to a folder, builds the new one, and passes both paths. The script then runs
A and B alternately for each case.

## Acceptance Criteria
- [ ] Invocations of two binaries alternate for one case
- [ ] Each side is written to its own .ns file that bench-compare reads

## Design Notes
- No git worktree is used (EPIC-067 and the disk limit); the owner builds the baseline from its own checkout.
- The paired comparison is what the regression rule assumes. Until this exists, the new side is compared against
  a saved baseline from another session, which the rule treats as unpaired.

## Dependencies
STORY-0201, TASK-0278.

## Open Questions
- Where the baseline binary is kept between releases.
