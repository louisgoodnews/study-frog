# StudyFrog 0.8.0

FrogScript standalone domain-specific language engine, ScriptStudio multi-tab automation workbench with in-vault script persistence, dedicated stdout and stderr stream execution logging with filter controls, and full 14-event reactive application lifecycle event bus. The full list of changes is in `CHANGELOG.md` under `[0.8.0]`.

## What is new

- **FrogScript Domain-Specific Language (`crates/studyfrog-script`).**
  - Standalone workspace crate with pure Rust isolated lexer, AST parser, and tree-walk evaluator.
  - Python-like syntax featuring first-class pipeline queries (`|`), variable assignment, duration literals (`30s`, `15m`, `2h`, `7d`), and statement chaining via semicolons (`;`).
  - Native query verbs (`from stack "..."`, `where tag "..."`, `set priority "..."`) with domain predicates (`tag`, `type`, `difficulty`, `priority`).
  - First-class closures (`{ item -> ... }`) for inline card and item inspection and mutation.
  - Native runtime globals and stdlib: `print`, `log_info`, `log_warn`, `log_error`, `len`, `now`, `today`, `days_ago`.
- **ScriptStudio Multi-Tab Workbench (`crates/studyfrog-ui`).**
  - Interactive multi-tab development environment with concurrent open tabs, draft editing, tab switching, and closing.
  - File persistence: discover, open, edit, and save `.frog`, `.py`, and `.sh` scripts directly in `.studyfrog/scripts/` (vault level) or application scripts folder (`<app home>/scripts/`).
  - In-app execution bridge: executes FrogScript against active vault items in memory with live stdout/stderr capture and lifecycle notifications.
- **Split Console Output Streams & Filtering.**
  - Independent stdout and stderr execution capture buffers.
  - Stream-level filter controls (`All`, `Stdout`, `Stderr / Errors`, `Events`) for isolating execution output from errors or lifecycle events.
- **Expanded Application Lifecycle Event Bus.**
  - Expanded from 6 events to 14 reactive lifecycle events covering all vault mutations, rehearsal steps, exams, focus blocks, imports, and backup runs (`vault_opened`, `vault_closed`, `item_created`, `item_saved`, `item_deleted`, `rehearsal_started`, `rehearsal_stepped`, `rehearsal_finished`, `exam_finished`, `diary_entry_saved`, `import_finished`, `backup_created`, `focus_block_finished`, `script_finished`).

## Verification

- **Automated quality gates:** All repository checks in `scripts/check.sh` pass cleanly on release commit:
  - Formatting (`cargo fmt --all --check`)
  - Tool linters (`repo-lint`)
  - Strict compiler lints (`cargo clippy --workspace --all-targets -- -D warnings`)
  - Full test suite across all 22 workspace crates
  - Line count guardrail ($\le 250$ lines per `.rs` file)
  - Layering and dependency direction (`check-deps.sh`)
  - Architecture guardrail (zero inherent `impl` blocks across all crates)
  - Issue metadata validation
  - Board consistency

## Install

StudyFrog is for Linux on x86_64. It requires OpenGL, and X11 or Wayland, from your system.

### AppImage

```sh
sha256sum -c StudyFrog-0.8.0-x86_64.AppImage.sha256
chmod +x StudyFrog-0.8.0-x86_64.AppImage
./StudyFrog-0.8.0-x86_64.AppImage
```

### Tarball

```sh
sha256sum -c StudyFrog-0.8.0-x86_64.tar.gz.sha256
tar -xzf StudyFrog-0.8.0-x86_64.tar.gz
cd StudyFrog-0.8.0
./bin/studyfrog
```

### Upgrading from 0.7.1

All existing vaults, notes, cards, and review histories remain fully compatible.
