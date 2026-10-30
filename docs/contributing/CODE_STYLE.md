# Code style

Authoritative rules are in [DESIGN_PRINCIPLES](../DESIGN_PRINCIPLES.md). Practical summary:

- `cargo fmt` (max width 100) and `cargo clippy -- -D warnings` must pass.
- Structs hold data only. Write `fn rate_card(card: &Card, r: Rating) -> Card`, not `card.rate(r)`.
- The only `impl` blocks allowed are trait implementations whose bodies delegate to free functions.
  Derives (`Debug, Clone, PartialEq, Serialize`) are fine.
- Max nesting depth 2 inside any function; extract a named helper at the third level.
- Prefer iterator pipelines and early returns over nested `if let`.
- Borrow by default: `&str`, `&[T]`, `Cow<'_, str>`. Clone only with a comment saying why.
- Errors: a `thiserror` enum per crate; no `unwrap`/`expect` outside tests.
- Public items have doc comments (`missing_docs` is on).
- File size: aim for 150 lines, never more than 250. One concept per file.
- Pass time, randomness and I/O in as parameters or trait objects at the edge.
- Naming: modules and functions `snake_case`, verbs for transforms (`schedule_next`, `parse_card`).
