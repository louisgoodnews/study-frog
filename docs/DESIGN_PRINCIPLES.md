# Design principles

These constraints are binding. A change that violates one needs an ADR first.

| # | Rule | Enforced by |
|---|---|---|
| P1 | **No OOP.** Structs are plain data: fields plus, if strictly necessary, a `new`-style constructor. No `impl` block gives a type behaviour. Behaviour is a free function taking `&T` or `&mut T`. | review + `scripts/check.sh` impl-block scan |
| P2 | **Functional orientation.** Free functions, pure transforms, explicit data flow, no hidden state. Prefer returning new values over in-place mutation where cheap. Time, randomness and I/O are passed in, never reached for. | review, `clippy::disallowed_methods` |
| P3 | **Data-oriented design.** Data and logic are separate. Use SoA where hot paths benefit. Use arenas and indices instead of `Rc<RefCell<>>` graphs. | review, benchmarks |
| P4 | **Nesting < 3.** A function body with three or more nested `if`/`match`/`for`/`loop` blocks is decomposed into named helpers. | review, `clippy::cognitive_complexity` |
| P5 | **Modular and generic.** Behaviour contracts are traits or generic functions. Reusable primitives become their own crates. | ADR 0001 |
| P6 | **File size.** Target about 150 lines, hard cap 250 per `.rs` file. If describing a file needs "and"/"or", split it. | `scripts/linecount.sh` |
| P7 | **Performance.** No premature clone; borrow or use `Cow`. No allocation in render loops. `smallvec`, `ahash`, `bumpalo` only where a benchmark justifies them. Profile before optimising. | `criterion`, [BENCHMARKING](contributing/BENCHMARKING.md) |
| P8 | **Dependency direction.** `app → ui → domain crates → core`. Domain crates never depend on `ui`. | ADR 0001, CI check |
| P9 | **Safety.** `unsafe` is forbidden workspace-wide; lifting it for one crate needs an ADR. | `unsafe_code = "forbid"` |
| P10 | **Build vs buy.** If we can write a better implementation than an external crate for our needs (smaller, faster, safer, or exactly fitted to our data model), we write it ourselves. A dependency must earn its place; record the comparison in an ADR or the task. Prefer well-maintained crates for hard, general problems (SQLite, GUI, ML runtimes, Markdown/CommonMark). | review, ADR |

## Trait boundaries

Traits are allowed where a contract has several implementations (SLM runtime, clock, store
backend, extension points). A trait declares function signatures only; implementations are
free-standing structs or closures, and trait `impl` blocks contain no logic beyond delegating
to a free function.

## Why trait impls are the one permitted `impl`

Rust requires an `impl Trait for Type` block to satisfy a trait. That is a binding, not
behaviour: the body is a single call into a free function. Everything else is P1.
