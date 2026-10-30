# M0: Bootstrap

**Goal:** Workspace, guardrails and documentation exist; nothing else.

**Entry criteria:** Repository exists with an initial commit.

## Epics, stories, tasks

- **EPIC-001** Workspace and guardrails
  - **STORY-0001** Workspace skeleton
    - TASK-0001 Convert to Cargo workspace
    - TASK-0002 Create crate skeletons with doc comments
    - TASK-0003 Dependency-direction check script
  - **STORY-0002** Guardrail scripts
    - TASK-0004 scripts/check.sh
    - TASK-0005 scripts/linecount.sh
    - TASK-0006 impl-block scan for P1
- **EPIC-002** Documentation library
  - **STORY-0003** Core docs and ADRs
    - TASK-0007 Vision, architecture, principles, glossary
    - TASK-0008 ADRs 0001-0005
  - **STORY-0004** Roadmap and issue tracking
    - TASK-0009 Roadmap and phase files
    - TASK-0010 Issue format and README
    - TASK-0011 Specs, research, contributing guides

## Exit criteria

- [ ] `cargo check --workspace` passes with all 12 crates
- [ ] `scripts/check.sh` runs fmt, clippy, test, line-count and exits 0 on the tree
- [ ] `scripts/check.sh` fails when a `.rs` file exceeds 250 lines (demonstrated)
- [ ] Dependency-direction check fails on a forbidden edge (demonstrated)
- [ ] All docs in the required tree exist; ADRs 0001-0005 written
- [ ] M0 issues seeded in `docs/issues/`

## Out of scope

Everything listed under later phases in [ROADMAP](ROADMAP.md).

## Risks

See ADRs and `STATUS.md` for open decisions affecting this phase.
