# FrogScript Language Specification (v1.0)

**Document:** `docs/specs/frogscript.md`  
**Related Issues:** [STORY-0149](../issues/stories/STORY-0149-rename-scriptrunner-to-avoid-trademark-conflicts.md), [STORY-0150](../issues/stories/STORY-0150-in-app-code-execution-console-and-dsl-research.md), [STORY-0151](../issues/stories/STORY-0151-scripting-ad-hoc-runs-scheduled-jobs-and-event-driven-triggers.md), [STORY-0152](../issues/stories/STORY-0152-dedicated-internal-stdout-and-stderr-execution-logging.md), [EPIC-049](../issues/epics/EPIC-049-expanded-scripting-environment-dsl-job-scheduling-and-execution-logging.md)  
**Author:** StudyFrog Architecture & Language Design Team  
**Status:** Draft / Specification  

---

## 1. Overview & Philosophy

**FrogScript** is StudyFrog's embedded Domain-Specific Language (DSL) designed for study automation, active-recall queue steering, deck generation, calendar scheduling, and custom Markdown macro authoring.

### Design Principles
1. **Pythonic Familiarity:** Clean, expressive syntax with readable operators, dynamic typing, and standard control structures (`if`, `for`, `def`).
2. **First-Class Pipeline Queries (`|`):** Natural stream piping from queries directly into transformations, mutations, or exporters without verbose loops.
3. **Ergonomic Closures:** Inline `{ item -> action }` lambda blocks for map/filter/mutate workflows over collections.
4. **Statement Chaining:** Support for semicolons (`;`) to concatenate statements on a single line for compact console execution.
5. **Native Study Primitives:** First-class knowledge domain types (`card`, `question`, `note`, `stack`, `due`, `familiarity`, `fsrs`, `timetable`, `diary`).
6. **Integrated Stream Logging:** Native `log_info`, `log_warn`, and `log_error` routines mapped directly to dedicated `stdout` and `stderr` channels in ScriptStudio.

---

## 2. Lexical Structure

### 2.1 Comments
Both single-line Python-style (`#`) and C-style (`//`) comments are supported:
```frogscript
# This is a comment
// This is also a comment
```

### 2.2 Semicolons & Statement Separation
Newlines separate statements. Semicolons (`;`) can optionally terminate statements or chain multiple statements onto a single line:
```frogscript
x = 10; y = 20; log_info(f"x + y = {x + y}")
```

### 2.3 Literals & Data Types
- **Numbers:** Integers (`42`), floats (`3.14`), durations (`30s`, `15m`, `2h`, `7d`).
- **Strings:** Quoted text (`"hello"`, `'world'`) with `f"..."` format string interpolation:
  ```frogscript
  name = "Biology"
  log_info(f"Loaded stack: {name}")
  ```
- **Booleans & Null:** `true`, `false`, `null` (or `None`).
- **Lists:** `[1, 2, 3, "apple"]`.
- **Maps / Dictionaries:** `{"priority": "high", "weight": 0.85}`.

---

## 3. Pipeline Expressions (`|`)

The pipe operator (`|`) passes the evaluated value of the left expression as the implicit input context to the right-hand operation.

### 3.1 Pipeline Syntax
```frogscript
<expression> | <verb> [arguments...]
```

### 3.2 Query Verbs

| Verb | Usage | Description |
| :--- | :--- | :--- |
| `from` | `from stack "Name"` / `from vault` | Sets source collection of study items. |
| `where` | `where <condition>` | Filters items matching boolean expression. |
| `set` | `set <property> <value>` | Mutates item properties (`priority`, `difficulty`, `status`). |
| `tag` | `tag "<tag_name>"` | Appends one or more tags to items. |
| `untag` | `untag "<tag_name>"` | Removes a tag from items. |
| `sort` | `sort <property> [asc\|desc]` | Reorders items by property (e.g. `due`, `familiarity`). |
| `limit` | `limit <n>` | Truncates collection to first $n$ items. |
| `select` | `select title, due` | Projects collection into tuples or map structures. |
| `export` | `export note "<path>"` | Exports piped items into a summary note or CSV. |

### 3.3 Variable Assignment & Re-Piping
Pipelines evaluate to item collections that can be stored in variables:
```frogscript
# Query and store in variable
urgent = from stack "Neuroscience" | where due <= today and familiarity < 0.6

# Continue pipeline from variable
urgent | set priority "high" | tag "cram-session"
```

---

## 4. Closures & Block Transformations

Closures provide fine-grained element-by-element inspection and conditional mutation.

### 4.1 Single-Line Arrow Closures
```frogscript
from stack * | where type "card" | {
    card -> if card.difficulty == "low" | set priority "low"
}
```

### 4.2 Multi-Line Closures
```frogscript
from stack "Language/French" | {
    card ->
        if card.familiarity < 0.4:
            card | set priority "high" | tag "needs-repetition"
            log_warn(f"Card '{card.title}' has low familiarity ({card.familiarity})")
        else:
            card | untag "needs-repetition"
}
```

### 4.3 Indexed Closures
Closures can accept an optional second argument representing index:
```frogscript
cards | { (card, i) -> log_info(f"[{i + 1}] Processing {card.title}") }
```

---

## 5. Built-In Globals & Standard Library

### 5.1 Stream Logging (`STORY-0152`)
FrogScript provides dedicated logging primitives routed to ScriptStudio's stream viewer:

| Function | Destination | Semantics |
| :--- | :--- | :--- |
| `print(*args)` | `stdout` | Standard print stream, space-separated stringification. |
| `log_info(msg)` | `stdout` | Informational execution progress in standard body font. |
| `log_warn(msg)` | `stdout` (warn) | Warning diagnostics in amber accent. |
| `log_error(msg)` | `stderr` | Error traces and failure alerts in rose/danger palette with line markers. |

```frogscript
log_info("Starting deck normalization...")
if not vault.has_stack("Chemistry"):
    log_error("Stack 'Chemistry' not found!")
    exit(1)
```

### 5.2 Date & Time Globals
- `today`: Civil date of current system day (`Date`).
- `tomorrow`, `yesterday`: Relative civil dates.
- `now`: Monotonic timestamp / current civil minute.
- Date Arithmetic: Supports adding and subtracting durations:
  ```frogscript
  due_cutoff = today + 3d
  stale_date = today - 14d
  ```

### 5.3 Domain Singletons

#### `vault`
- `vault.stacks -> List[Stack]`
- `vault.has_stack(name: str) -> bool`
- `vault.stack(name: str) -> Stack`
- `vault.create_card(stack: str, title: str, front: str, back: str, difficulty="Hard", priority="Medium") -> Card`
- `vault.tags -> List[str]`

#### `calendar` & `timetable`
- `timetable.add_entry(course: str, weekday: str, start: str, duration: int, room: str, stack: str)`
- `calendar.schedule_study(title: str, date: str, start: str, duration: int, stack: str)`
- `calendar.upcoming_exams(days=30) -> List[Exam]`

#### `diary`
- `diary.entry(title: str, stack: str, duration_min: int, ratings: Map, body: str)`

#### `run` (Active Rehearsal Context)
When triggered during rehearsal runs or `on rehearsal_finished`:
- `run.accuracy -> float` (0.0 to 1.0)
- `run.item_count -> int`
- `run.duration_min -> float`
- `run.median_latency_ms -> int`
- `run.stack_name -> str`

---

## 6. Control Flow & Custom Functions

### 6.1 Conditionals
```frogscript
if familiarity < 0.5:
    priority = "high"
elif familiarity < 0.8:
    priority = "medium"
else:
    priority = "low"
```

### 6.2 Loops
```frogscript
for stack in vault.stacks:
    due_count = len(from stack stack.name | where due <= today)
    log_info(f"Stack {stack.name}: {due_count} due")
```

### 6.3 Function Declarations
```frogscript
def prioritize_deck(stack_name, min_familiarity=0.6):
    items = from stack stack_name | where familiarity < min_familiarity
    items | set priority "high"
    log_info(f"Updated {len(items)} items in {stack_name}")
```

### 6.4 Markdown Macro Authoring
Users can register custom Markdown macros expanded in notes and cards:
```frogscript
macro formula_callout(name, formula):
    return f"> [!NOTE] **{name}**\n> $${formula}$$\n"

macro quick_quiz(question, answer):
    return f"**Q:** {question}\n\n{{hide:{answer}}}"
```

---

## 7. Execution Architecture & Sandboxing

### 7.1 Parser & Evaluator Integration
- **Zero-Dependency Evaluator:** The FrogScript parser and tree-walk / bytecode interpreter is implemented in pure Rust within `crates/studyfrog-service` and `crates/studyfrog-ui`.
- **Strict P1 & P2 Compliance:** Modular architecture with no inherent `impl` blocks and every `.rs` file kept under 250 lines.

### 7.2 Safety & Resource Limits
- **Step Cap:** Scripts are constrained to a default execution quota of 100,000 operations per run to prevent infinite loops.
- **Memory Ceiling:** String and list allocations are bounded to 16 MB per script invocation.
- **Non-Blocking UI:** Long-running scripts run on worker threads or background channels, streaming `stdout`/`stderr` chunks asynchronously into ScriptStudio.

---

## 8. Example Script: Complete Study Maintenance

```frogscript
# ScriptStudio FrogScript: Automated Study Maintenance
log_info("Executing automated morning maintenance...")

# 1. Promote due cards with low familiarity
urgent = from stack * | where due <= today and familiarity < 0.55
if len(urgent) > 0:
    log_warn(f"Found {len(urgent)} high-risk cards due today. Raising priority.")
    urgent | set priority "high" | tag "focus-today"

# 2. Reset priority on mastered cards
from stack * | where familiarity >= 0.90 | {
    card -> if card.priority == "high" | set priority "low" | untag "focus-today"
}

# 3. Check upcoming exams within 7 days
exams = calendar.upcoming_exams(days=7)
for exam in exams:
    log_info(f"Upcoming exam: {exam.title} on {exam.date}")
    from stack exam.stack | where state != "mature" | tag "exam-priority"

log_info("Maintenance complete.")
```
