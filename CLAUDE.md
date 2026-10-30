# StudyFrog: working rules

Read `docs/DESIGN_PRINCIPLES.md` first. Hard constraints: no OOP (no inherent `impl` blocks),
functional free functions, nesting < 3, files <= 250 lines (aim ~150), performance by measurement.

## Always, on every change
1. **Board:** set the work item's `status:` in `docs/issues/**` when work starts (`in_progress`),
   moves to review, is blocked, or finishes (`done`); then run `scripts/board.sh` and commit
   `BOARD.md` in the same commit. Never edit `BOARD.md` by hand. WIP limit is 3.
2. **Changelog:** add a line under `## [Unreleased]` in `CHANGELOG.md` for every user- or
   developer-visible change (Added / Changed / Fixed / Removed), naming the `TASK-xxxx`.
3. **Gate:** `scripts/check.sh` must pass before committing.
4. Conventional commits with a `Refs: TASK-xxxx` footer (see `docs/contributing/COMMIT_CONVENTIONS.md`).
5. Update `STATUS.md` (next task, open flags) when the active task changes.

## Build vs buy (P10)
If you can build a better implementation than an external crate for our needs, build it. Keep crates
for hard general problems (SQLite, GUI, ML runtime, CommonMark). Note the comparison in the task.

## Where things are
Roadmap `docs/roadmap/ROADMAP.md`; issues `docs/issues/` (format in its README); ADRs `docs/adr/`;
specs `docs/specs/`.
