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

## Looking at the real window
Headless tests (`egui_kittest`) check structure, not looks. To see the window, run a debug build on X11 with a
application home (`STUDYFROG_HOME`) whose `config/app.toml` names a small vault (`vault = "/path/to/vault"`), and capture the screen while
it runs: `env -u WAYLAND_DISPLAY WINIT_UNIX_BACKEND=x11 STUDYFROG_HOME=/tmp/cfg timeout 8 target/debug/studyfrog
--smoke-frames 5000`, and from another shell after a few seconds `convert x:root shot.png`. Run one window at a
time: a leftover window overlaps in the capture and looks like duplicated widgets. The bundled egui fonts have no
emoji and egui panics on a missing glyph, so icons are drawn with shapes, not characters.
