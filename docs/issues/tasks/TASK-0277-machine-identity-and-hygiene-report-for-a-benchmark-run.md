---
id: TASK-0277
type: task
title: "Machine identity and hygiene report for a benchmark run"
status: done
epic: EPIC-067
parents: ['STORY-0201']
estimate: 1
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'scripts/bench-machine.sh prints the CPU model, logical CPUs, kernel, rustc, commit, dirty files, governor, boost, SMT, load and busy processes as key=value lines'
  - 'It warns on stderr for each hygiene item not met (governor, boost, busy processes, uncommitted files) and records neither hostname nor user name'
---

## Context
Part of STORY-0201. A result is comparable only with the machine and state it came from.

## Description
`scripts/bench-machine.sh <label>` is read-only. It reads sysfs and /proc, and the cpufreq and boost settings
are reported as found; the owner sets them with root before a run.

## Acceptance Criteria
- [x] scripts/bench-machine.sh prints the identity and hygiene fields as key=value lines
- [x] It warns for each hygiene item not met, and records neither hostname nor user name

## Design Notes
- Checked here: the script ran in this container and printed every field. It warned for governor, boost and
  uncommitted files, as it should. The container has no cpufreq controls, so the warnings are expected here.
- Intel pstate `no_turbo` is read when present; otherwise the generic cpufreq `boost` flag.

## Dependencies
STORY-0201.

## Open Questions
None.
