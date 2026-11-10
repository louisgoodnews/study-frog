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
