# ADR 0014: Decisions for the 1.0.0 release

- Status: **Accepted** (2026-11-24, owner)
- Date: 2026-11-24

## Context
Milestone 14.5 of the v1.0.0 roadmap (`docs/v1.0.0-roadmap.md`, `priority-list.md`) asks for a regression
audit, packaging (AppImage and tarball), documentation, and the release tag `v1.0.0`. The regression audit
(`docs/RELEASE_AUDIT_1.0.0.md`) left four decisions to the owner, because models could not take them alone:

1. whether the stories and epics still in review are part of 1.0.0 scope,
2. whether the reference benchmark runs (EPIC-067, `blocked` on "the owner's reference machine") can run on the
   machine this work is done on,
3. whether a release tag may be pushed,
4. where the release is published.

The audit also recorded that the AppImage was not produced because `desktop-file-validate` was missing. That
package is now installed.

## Decision
The owner made these decisions on 2026-11-24:

1. **Scope.** Every story and epic in the `review` state is part of 1.0.0. The release does not wait for them,
   and they are not held back from the tag. The list is in `docs/RELEASE_AUDIT_1.0.0.md`, section 3.
2. **Reference machine.** The machine at `/home/louisgoodnews` is the reference machine for EPIC-067. Its
   benchmark runs can be done here, and the machine's identity (CPU, memory, OS, toolchain) is recorded with
   each run (STORY-0201). The items blocked on the owner's machine (STORY-0201, STORY-0203) are unblocked by this
   decision.
3. **Tag and push.** The annotated tag `v1.0.0` is pushed to `origin` once the release has been verified, that
   is, after the gate, the AppImage build, the desktop-start check and the benchmark run have passed. The push
   publishes the GitHub release through `.github/workflows/release.yml`. This authorisation covers the v1.0.0
   push only. It does not cover later releases or other pushes.

Still open, not decided here: the owner's window checks in `RELEASE_AUDIT_1.0.0.md` section 3 (not run by the
release work unless a running display is available) and the desktop start of the AppImage on the owner's own
desktop session (RELEASING step 6).

## Consequences
- The release work may be done by models, within the limits above. The release agent creates the commits and
  the local tag, then pushes the tag only after the checks in item 3 pass.
- `RELEASING.md` steps 2 and 3 are run here and their results are recorded in the audit, not left as "owner's
  machine only".
- Scope decisions for 1.0.0 are not reopened without a new ADR.

## Related
- ADR 0013 (scripting out of process) and the release process in `docs/contributing/RELEASING.md`.
- `docs/RELEASE_AUDIT_1.0.0.md`, `docs/RELEASE_NOTES_1.0.0.md`.

## Addendum (2026-11-24): reference baseline for 1.0.0

The owner decided on 2026-11-24, after the release benchmark check reported `regression` for 16 to 17 of 28 cases
against the first reference run:

> Re-baseline the 1.0.0 reference on runs 2 to 4. Run 1 is recorded as contaminated by load (14 busy processes during
> the run); runs 2 to 4 are the reference.

Reason: run 1 ran with 14 busy processes (load 3.0, governor powersave); runs 2 to 4 ran with one busy process and agree
with each other on all 28 cases. The earlier decisions above are unchanged. Run 1 stays in `docs/benchmarks/reference-1.0.0/`
as recorded data, but it is not the baseline for the 1.0.0 comparison. The spread-rule cases that remain above 5% after
10 runs are recorded as known noise in `docs/RELEASE_AUDIT_1.0.0.md`, not as passes.
