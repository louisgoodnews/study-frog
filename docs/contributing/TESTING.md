# Testing

- **Unit tests** beside code (`#[cfg(test)] mod tests`), counted in the 250-line cap; move large
  suites to `tests/` integration files.
- **Property tests** (`proptest`) are mandatory for: SRS (monotonicity, bounds), scoring
  (range, identity, whitespace invariance), scheduling/recurrence, Markdown round-trip
  (`parse(render(x)) == x`), board moves (WIP invariant never violated).
- **Golden tests** for FSRS reference vectors and front-matter fixtures in `tests/fixtures/`.
- **Integration tests** use temp dirs and in-memory SQLite; no test touches the user's vault.
- Domain crates are pure, so tests need no mocks: inject a fake `Clock` value or parameter.
- Run everything with `scripts/check.sh`. Failing proptest seeds are committed as regression files.
- UI is covered by testing `update(State, Msg)` as pure functions; rendering is smoke-tested only.
