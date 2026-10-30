# Commit conventions

Conventional Commits: `type(scope): summary` (imperative, under 72 chars).

Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `perf`, `test`, `build`, `ci`.
Scope is the crate short name (`srs`, `md`, `ui`) or `docs`/`repo`.

- One logical change per commit; keep the build green at every commit.
- Reference work items in the footer: `Refs: TASK-0016`, `Closes: STORY-0007`.
- Breaking changes: `!` after the type and a `BREAKING CHANGE:` footer.
- Performance commits include before/after numbers.
