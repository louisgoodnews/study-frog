# ScriptRunner Environment & Internal Automation API Specification

**Issue:** [STORY-0148](../issues/stories/STORY-0148-console-expansion-to-full-scriptrunner-environment-with-internal-api.md) / [EPIC-048](../issues/epics/EPIC-048-scriptrunner-environment-and-internal-automation-api.md)  
**Author:** StudyFrog Architecture & Extensibility Team  
**Date:** 2026-11-10  
**Status:** Completed  

---

## 1. Overview & Vision

StudyFrog is designed as an extensible, hackable learning workbench. While previous milestones delivered CLI-based scripting (`studyfrog script run <job>`, ADR 0013) and the background control socket (`~/.local/share/studyfrog/run/control.sock`), users requested an **in-app ScriptRunner environment** with an ergonomic internal Python API.

The ScriptRunner environment allows learners to:
1. Write, test, and execute Python automation scripts directly within the StudyFrog GUI.
2. Automate repetitive deck operations (e.g., batch generation of flashcards from lecture notes, syllabi, or CSVs).
3. Query vault items, review schedules, and learning analytics.
4. Programmatically schedule calendar study sessions and timetable lecture blocks.
5. Register bespoke text and preview macros (e.g., custom mathematical formatting, callout styling, data visualizers).

---

## 2. In-App ScriptRunner Architecture

The ScriptRunner integrates the UI Console (`crates/studyfrog-ui/src/view/console.rs`) with the underlying IPC control protocol (`studyfrog-ui::control_client`, `studyfrog-app::script_runner`):

```
┌────────────────────────────────────────────────────────┐
│               StudyFrog GUI Console                    │
│  ┌────────────────────────┐  ┌───────────────────────┐ │
│  │ Multi-Line Code Editor │  │  Live Log & Stdout/   │ │
│  │ (Python Script/Snippet)│  │  Stderr Output Stream │ │
│  └───────────┬────────────┘  └───────────▲───────────┘ │
└──────────────┼───────────────────────────┼─────────────┘
               │ "script run" / IPC stream │
┌──────────────▼───────────────────────────┴─────────────┐
│             StudyFrog Control Protocol Engine          │
│               (Local Socket / Stdio IPC)               │
└──────────────────────────────┬─────────────────────────┘
                               ▼
┌────────────────────────────────────────────────────────┐
│           Python Runtime (`studyfrog.app`)             │
│   • studyfrog.vault      • studyfrog.calendar          │
│   • studyfrog.rehearsal  • studyfrog.macros            │
└────────────────────────────────────────────────────────┘
```

---

## 3. The `studyfrog` Python API Specification

The built-in Python client package `studyfrog` (provided in the application distribution) exposes the following high-level automation modules:

### 3.1 Vault & Items (`studyfrog.vault`)

```python
import studyfrog as sf

# Connect to the active StudyFrog window session
app = sf.connect()

# Query items using StudyFrog query language
cards = app.vault.query("type = card and status = live and tag = chemistry")
print(f"Found {len(cards)} active chemistry cards")

# Batch create flashcards in a stack
with app.vault.batch() as batch:
    for term, definition in terms_dict.items():
        batch.create_card(
            stack="Biology/Cellular",
            title=term,
            front=f"What is the function of **{term}**?",
            back=definition,
            difficulty="Hard",
            priority="Medium"
        )
```

### 3.2 Calendar & Timetable (`studyfrog.calendar`)

```python
# Create or update a weekly recurring lecture
app.calendar.timetable.add_entry(
    title="Computer Systems Architecture",
    day_of_week=sf.Weekday.Wednesday,
    start_time="10:15",
    duration_min=90,
    location="Lecture Hall 3",
    instructor="Prof. Tanenbaum",
    stack="Computer Science/Architecture"
)

# Plan a one-off rehearsal or study block
app.calendar.schedule_study(
    title="Exam Prep: Architecture Review",
    date="2026-11-15",
    start_time="14:00",
    duration_min=120,
    stack="Computer Science/Architecture"
)
```

### 3.3 Custom Text & Markdown Macros (`studyfrog.macros`)

```python
@app.macro.register("formula_callout")
def formula_callout(name: str, formula: str) -> str:
    """Expands {formula_callout:name, formula} into a formatted alert block."""
    return f"> [!NOTE] **{name}**\n> $${formula}$$\n"

@app.macro.register("quick_quiz")
def quick_quiz(question: str, answer: str) -> str:
    """Expands {quick_quiz:question, answer} into an interactive spoiler."""
    return f"**Q:** {question}\n\n{sf.macro.hide(answer)}"
```

---

## 4. Security & Isolation Model

1. **Local-Only Boundary:** The control socket and script execution operate strictly under user permissions with Unix domain sockets (`~/.local/share/studyfrog/run/control.sock`) and local process execution.
2. **Deterministic Output Capture:** Scripts stream stdout and stderr line-by-line into the Console transcript with millisecond timestamps and color-coded error spans (`pal.danger`).
3. **Execution Timeouts:** Scripts default to a 30-second execution cap to prevent runaway loops from freezing user workflows.

---

## 5. Rebranding to FrogScript Studio (Trademark Safety)

> **Context (`STORY-0149`):** Adaptavist holds registered trademarks on "ScriptRunner" across software automation, IDE, and developer ecosystem tooling. 

To eliminate trademark exposure and cultivate a distinct StudyFrog identity, the in-app scripting workbench is rebranded to **FrogScript Studio** (or **FrogScript**):
- **User Interface:** UI headers, tabs, and action buttons transition from `"ScriptRunner Editor"` and `"Hide ScriptRunner"` to `"FrogScript Studio"`, `"Run FrogScript"`, and `"FrogScript Console"`.
- **Identifiers & Storage:** Configuration keys and widget salt IDs transition cleanly to `frogscript-code-buffer` and `frogscript-multiline-text`.
- **CLI & Sockets:** User-facing documentation refers to the scripting runtime as the FrogScript engine.

---

## 6. In-App Scripting Console & Domain-Specific Language (DSL) Research

> **Context (`STORY-0150`):** Users need to run scripts directly within StudyFrog without installing an external Python interpreter or configuring environment variables.

### 6.1 Architectural Trade-Off Analysis

| Criteria | External Python IPC (Current) | Embedded DSL (Rhai / Custom) | Hybrid Model (Recommended) |
|---|---|---|---|
| **Prerequisites** | Requires Python 3 on host system (`/usr/bin/python3`) | Zero dependencies; compiled directly into Rust binary | Embedded DSL for immediate in-process automation; optional Python IPC for complex packages |
| **Startup Latency** | ~50–120 ms (process fork & socket handshake) | < 1 ms (direct in-process evaluation) | Instant for DSL queries; asynchronous for Python tasks |
| **Sandboxing & Safety** | Operating system process permissions | Strict AST evaluation limits; zero raw OS / filesystem access without explicit host hooks | Safe-by-default embedded scripts; explicit consent for Python access |
| **Ecosystem & Libraries**| Vast ecosystem (NumPy, SciPy, Pandas, PyTorch) | Limited to language primitives & StudyFrog bindings | Best of both worlds: quick scripts use DSL; scientific/AI flows use Python |

### 6.2 DSL Candidate Evaluation: Rhai vs. Custom Query DSL
1. **Rhai (`rhai.rs`):** An embedded scripting language designed specifically for Rust applications with JavaScript-like syntax. Offers fine-grained control over execution steps (`max_operations`), memory limits, and seamless exposure of Rust structs without `unsafe`.
2. **Custom StudyFrog Query/Action DSL:** An extension of StudyFrog's SQL-like query language supporting actions (e.g., `TAG "biology" WHERE due < TODAY AND difficulty = "Hard"`).

---

## 7. Execution Modes: Ad-Hoc Runs, Scheduled Cron Jobs, and Event-Driven Triggers

> **Context (`STORY-0151`):** Automation needs flexible execution strategies beyond single manual runs.

1. **Ad-Hoc Interactive Runs:** Executed directly from the FrogScript Studio editor by clicking "Run FrogScript" or pressing `Ctrl+Enter`. Output streams immediately to the execution viewer.
2. **Scheduled Recurring Cron Jobs:** Defined in `scripts.toml` and managed via UI:
   - Presets: `@hourly`, `@daily`, `@weekly`, or standard 5-field cron syntax (`0 7 * * mon-fri`).
   - Use Cases: Nightly deck backups, morning study queue preparation, periodic timetable calendar synchronization.
3. **Event-Driven Triggers:** Automated reactive scripts bound to application lifecycle hooks:
   - `vault_opened`: Verify vault structure, initialize study tags, sync external calendars.
   - `item_created`: Auto-enrich card front-matter, compute readability scores, link references.
   - `rehearsal_finished`: Update diary reflections, analyze fatigue metrics, alert on leech formation.

---

## 8. Dedicated Internal stdout and stderr Execution Logging

> **Context (`STORY-0152`):** Effective script authoring requires clean diagnostic separation between standard application output and runtime errors.

1. **Stream Separation:**
   - **`stdout` Channel:** Standard print statements, returned data representations, and query outputs formatted in standard monospace body typography.
   - **`stderr` Channel:** Python tracebacks, compiler/evaluator warnings, and error diagnostics highlighted in alert/danger tones (`pal.danger`).
2. **Stream Multiplexing & Metadata:** Each captured log chunk is tagged with monotonic timestamp offsets, stream origin (`stdout` / `stderr`), and job identifier.
3. **Stream Filtering & Export:** Console UI provides toggle filters (`All`, `Output`, `Errors`), text search, and single-click export to `<vault>/logs/frogscript_runs.log`.
