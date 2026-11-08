# StudyFrog Planned Features (Roadmap — Phase M12, carried past v0.5.0)

**After 0.5.0:** part of this scope was released in `v0.5.0` and is described in **[FEATURES.md](FEATURES.md)** (prompts, achievements, Gemini and DeepSeek, the application home and logs, the CSV export, the MCP server in part, and the PDF fixes). Everything below that is not listed there is still planned. The table shows where each epic stands.

This document catalogs all planned features, epics, stories, and tasks that are currently in the backlog and have not yet been implemented. For features already available in the current release (`v0.5.0`), see **[FEATURES.md](FEATURES.md)**. For live issue statuses, see **[BOARD.md](BOARD.md)**.

---

## Overview of Planned Epics

| Epic | Title | Child Stories / Tasks | Target |
| :--- | :--- | :--- | :--- |
| **[EPIC-028](docs/issues/epics/EPIC-028-extensibility-plugin-modding-and-python-api.md)** | Extensibility, Plugin Modding, and Python API | `STORY-0083`, `STORY-0084`, `STORY-0085`, `BUG-0010` | Gemini, DeepSeek and the PDF fixes in `0.5.0`; Python API planned |
| **[EPIC-029](docs/issues/epics/EPIC-029-expanded-achievements-and-per-stack-progression.md)** | Expanded Achievements and Per-Stack Progression | `STORY-0086`, `STORY-0087` | **Released in `0.5.0`** |
| **[EPIC-030](docs/issues/epics/EPIC-030-prompt-expansion.md)** | Prompt Expansion & Interactive AI Workflows | `STORY-0088`, `STORY-0089`, `STORY-0090`, `STORY-0091`, `STORY-0092`, `TASK-0233` | **Released in `0.5.0`** |
| **[EPIC-032](docs/issues/epics/EPIC-032-item-keys-colorization-and-reporting-export.md)** | Item Keys, Colorization, and Reporting Export | `STORY-0096`, `STORY-0097`, `STORY-0098`, `STORY-0123` | `STORY-0098` in `0.5.0`; the rest planned |
| **[EPIC-033](docs/issues/epics/EPIC-033-exam-run-mode-with-semantic-comparison-and-grading.md)** | Exam Run Mode with Semantic Comparison and Grading | *Epic-level specification* | Planned |
| **[EPIC-034](docs/issues/epics/EPIC-034-behavioral-learning-analytics-fsrs-optimizer-and-workload-forecasting.md)** | Behavioral Learning Analytics, FSRS Optimizer, and Workload Forecasting | `STORY-0102`, `STORY-0103`, `STORY-0104`, `STORY-0105`, `STORY-0106`, `STORY-0107` | Planned |
| **[EPIC-035](docs/issues/epics/EPIC-035-rich-active-recall-formats-inline-note-extraction-and-focus-workflows.md)** | Rich Active-Recall Formats, Inline Note Extraction, and Focus Workflows | `STORY-0108`, `STORY-0109`, `STORY-0110`, `STORY-0111`, `STORY-0112` | Planned |
| **[EPIC-036](docs/issues/epics/EPIC-036-evidence-based-ui-aesthetics-perceptual-color-science-and-ux-refinement.md)** | Evidence-Based UI Aesthetics, Perceptual Color Science (OKLCH/APCA), and UX Refinement | `STORY-0113`, `STORY-0114`, `STORY-0115`, `STORY-0116` | Planned |
| **[EPIC-037](docs/issues/epics/EPIC-037-state-of-the-art-model-context-protocol-mcp-server-client-and-registry.md)** | State-of-the-Art Model Context Protocol (MCP) Server, Client Host, and Registry | `STORY-0117`, `STORY-0118`, `STORY-0119`, `STORY-0120`, `STORY-0121` | Engine, read-only resources and stdio in `0.5.0`; the rest planned |
| **[EPIC-038](docs/issues/epics/EPIC-038-cli-control-protocol-for-interactive-ai-agent-application-operation.md)** | CLI Control Protocol for Interactive AI Agent Operation & `~/.local/` App Files | `STORY-0122`, `STORY-0124` | `STORY-0122` in `0.5.0`; `STORY-0124` planned |

---

## 1. Exam Run Mode with Semantic Comparison & Grading ([EPIC-033](docs/issues/epics/EPIC-033-exam-run-mode-with-semantic-comparison-and-grading.md))

While StudyFrog's existing `Flip cards` and `Write answers` rehearsal modes are designed for formative, self-paced practice with manual rating overrides and skipping, **Exam Run Mode** introduces a strict summative exam simulation for learners preparing for university module exams.

### Planned Capabilities
- **Strict One-Way Progression:**
  - Selectable as an **Exam** mode in the Rehearsal setup screen (`RunMode::Exam`).
  - Prohibits moving backward to previous items, peeking ahead, or skipping items (`Skip` is disabled).
  - Prohibits advancing to the next item until a non-empty written answer has been submitted.
  - Locks each answer upon submission without allowing manual self-rating overrides (`Again` / `Hard` / `Good` / `Easy`).
- **Mandatory Written Answers & Semantic Comparison:**
  - Requires typed text entry for both **Flashcards** (`Doc::Card`, evaluated against the card's `back`) and **Questions** (`Doc::Question`, evaluated against `reference`, `accepted` synonyms, and `rubric`).
  - Automatically scores each submitted answer using StudyFrog's semantic comparison engine (`studyfrog-slm` / token & embedding similarity) to produce an objective per-item score in `[0.0, 1.0]` (`0%` – `100%`).
- **Standardized Percentage Grading Scale (`60%` Pass Threshold & `5%` Bands):**
  - Aggregates item scores into a final exam percentage and maps it to a combined international (`A+`–`F`) and German university (`1+`–`5.0`) grade scale:
    - **`95% – 100%`** (top `5%` threshold): **`A+` / `1+`** (*Sehr gut+* — Very Good)
    - **`90% – <95%`**: **`A` / `1.0`** (*Sehr gut*)
    - **`85% – <90%`**: **`A-` / `1.3`**
    - **`80% – <85%`**: **`B+` / `1.7`**
    - **`75% – <80%`**: **`B` / `2.0`** (*Gut*)
    - **`70% – <75%`**: **`B-` / `2.3`**
    - **`65% – <70%`**: **`C` / `3.0`** (*Befriedigend*)
    - **`60% – <65%`**: **`D` / `4.0`** (*Ausreichend* — minimum **`60%`** required to pass)
    - **`< 60%`**: **`F` / `5.0`** (*Nicht bestanden* — Failed)
- **Dedicated Exam Results & Review Breakdown:**
  - Displays a final percentage score, grade badge (`A+ / 1+` through `F / 5.0`), Pass/Fail banner, and a scrollable item-by-item comparison of the learner's written response, the reference answer, and the semantic similarity score.
  - Persists exam runs so results integrate with stack familiarity metrics and post-run Learning Diary reflections.

---

## 2. Item Keys, Colorization & Reporting Export ([EPIC-032](docs/issues/epics/EPIC-032-item-keys-colorization-and-reporting-export.md))

As vaults grow to hundreds or thousands of items across multiple stacks, learners need human-memorable identifiers, visual color coding, and portable analytics exports.

### 2.1 Sequential Human-Readable Item Keys ([STORY-0096](docs/issues/stories/STORY-0096-item-keys-for-querying.md))
- **Memorable Per-Kind Keys:** Assigns sequential, human-readable keys to every item and stack based on kind (`STACK_1`, `FLASHCARD_500`, `NOTE_12`, `QUESTION_3`) alongside internal 26-character ULIDs.
- **Query & Search Bar Integration:** Supports `key:FLASHCARD_500` predicates in the query language and direct key lookup in the top bar or library search input.
- **UI Badges & Copy Action:** Displays item keys prominently in the browser table, item header, and Details notebook, with a one-click `Copy key` button for cross-linking.

### 2.2 Item Colorization & Viewer Background Styling ([STORY-0097](docs/issues/stories/STORY-0097-item-colorization-with-color-picker-and-viewer-background-styling.md), [STORY-0123](docs/issues/stories/STORY-0123-colorization-field-for-items-influencing-details-and-rehearsal-background.md))
- **Front-Matter `color` Property:** Stores an optional hex color code or curated palette token (`color: "#4A90E2"`) in item and stack front-matter.
- **Color Picker in Editor & Details Properties (`STORY-0097`, `STORY-0123`):** Adds a color picker widget in the Editor and the Item Details `Properties` tab featuring curated pastel chips (tuned for both dark and light themes), custom hex selection, and a `Clear color` reset.
- **Tinted Details & Rehearsal View Backgrounds (`STORY-0123`):** Applies a subtle, contrast-safe background tint derived from the item's colorization directly to the **Item Details / Editor container** and the **Rehearsal card surface**, plus color indicators in the library table and folder tree.

### 2.3 Reporting Export as CSV ([STORY-0098](docs/issues/stories/STORY-0098-reporting-export-as-csv.md))
- **One-Click CSV Export:** Adds an `Export CSV` action to the Reports screen toolbar using a native file save dialog (`studyfrog-report-<vault>-<date>.csv`).
- **Comprehensive Telemetry Tables:** Exports RFC 4180 CSV tables covering overall summary metrics (reviews, learned/mastered counts, retention rate, streak), per-stack statistics, and timestamped rehearsal run history logs.

---

## 3. Prompt Library Expansion & Standalone Navigation ([EPIC-030](docs/issues/epics/EPIC-030-prompt-expansion.md))

Expands StudyFrog's static 6-prompt content generator (`flashcards`, `questions`, `stack`, `notes`, `summary`, `critique`) into a categorized, parameterized learning and tutoring suite with its own first-class sidebar navigation entry.

### 3.1 Multi-Level File Explanation Prompt ([STORY-0088](docs/issues/stories/STORY-0088-multi-level-file-explanation-prompt.md))
- Adds `docs/prompts/explain.md` instructing an AI model to explain an attached document at three selectable levels of abstraction:
  1. **Level 1 — Intuitive Analogy (ELI5):** Everyday analogies and plain language with zero assumed jargon.
  2. **Level 2 — Core Concepts & Mechanisms:** Standard terminology, component interactions, and practical significance.
  3. **Level 3 — Technical Deep-Dive:** Formal definitions, mathematical derivations, edge cases, and low-level mechanics.
- Produces structured Markdown notes tagged with `explain/intuitive`, `explain/core`, and `explain/technical`.

### 3.2 Interactive File Examination & Feedback Prompt ([STORY-0089](docs/issues/stories/STORY-0089-interactive-file-examination-and-feedback-prompt.md))
- Adds `docs/prompts/exam.md` instructing an AI to act as an oral/written examiner on an attached file, posing 3–5 comprehension and application questions.
- Evaluates the learner's responses against a rubric covering **Accuracy**, **Omissions & Misconceptions**, and **Model Answer & Elaboration**, then recommends targeted follow-up flashcards.

### 3.3 Socratic Dialogue & Active Inquiry Prompt ([STORY-0090](docs/issues/stories/STORY-0090-socratic-dialogue-and-active-inquiry-prompt.md))
- Adds `docs/prompts/socratic.md` enforcing a question-driven tutoring dialogue where the AI never lectures outright, asks one focused question at a time, challenges partial misconceptions with counter-examples, and concludes by synthesizing the learner's discoveries into an importable summary note.

### 3.4 Concept Mapping & Prerequisite Analysis Prompt ([STORY-0091](docs/issues/stories/STORY-0091-concept-mapping-and-prerequisite-analysis-prompt.md))
- Adds `docs/prompts/concept_map.md` to extract **Prerequisite Dependencies**, **Core Mental Models**, and a Markdown **Contrast Matrix** (contrasting easily conflated concepts) with `[[Concept Name]]` wikilinks that integrate directly into StudyFrog's link graph.

### 3.5 Categorized & Parameterized Prompt Library UI ([STORY-0092](docs/issues/stories/STORY-0092-prompt-library-categorization-and-parameterized-execution-ui.md))
- Organizes prompts into three categories: **Content Generation**, **Interactive Tutoring & Exam**, and **Analysis & Insights**.
- Adds interactive UI controls (chips/dropdowns) to interpolate parameters (such as explanation depth or question count) before copying, saving, or executing a prompt, plus search filtering.

### 3.6 Standalone Sidebar Navigation for Prompts ([TASK-0233](docs/issues/tasks/TASK-0233-move-prompt-library-to-standalone-left-sidebar-navigation.md))
- Promotes the Prompt Library from a sub-tab inside the Assistant view to a dedicated top-level screen (`Screen::Prompts`) accessible directly from the left navigation sidebar with full back/forward history integration.

---

## 4. Expanded Achievements & Per-Stack Progression ([EPIC-029](docs/issues/epics/EPIC-029-expanded-achievements-and-per-stack-progression.md))

Expands StudyFrog's built-in achievement catalog beyond the initial 8 rules and adds stack-scoped filtering and progress badges.

### 4.1 Expanded Achievement Catalog & Milestone Tiers ([STORY-0086](docs/issues/stories/STORY-0086-expanded-achievement-catalog-and-milestone-tiers.md))
- **Streak Tiers:** `3`, `7`, `14`, `30`, `60`, `100`, and `365` consecutive study days (*Getting into the rhythm* through *Year of study*).
- **Rehearsal Run Tiers:** `1`, `10`, `25`, `50`, and `100` completed runs (*First run* through *Century club*).
- **Rating & Volume Tiers:** `50`, `250`, `1,000`, and `5,000` rated items, plus `20` and `100` `Easy` ratings.
- **Global Mastery Tiers:** `10`, `50`, `100`, `250`, and `500` items mastered (`21+` day intervals, up to *Grandmaster*).
- **Per-Stack Progression Tiers:** `10`, `25`, `50`, and `100` items learned in a stack; `10`, `25`, `50`, and `100` items mastered in a stack; and `50` ratings in a stack.

### 4.2 Per-Stack Achievement UI & Filtering ([STORY-0087](docs/issues/stories/STORY-0087-per-stack-achievement-ui-and-filtering.md))
- **Scope & Stack Filter Bar:** Filter the Achievements screen by `All`, `Global`, or a specific `[Stack Name]`, with visual grouping by category (`Streaks`, `Runs`, `Cards`, `Mastery`).
- **Stack Workspace Badge:** Displays an achievement summary pill (e.g., `3 / 9 achievements unlocked`) directly on the Stack workspace overview.

---

## 5. Extensibility, Python API, AI Providers & PDF Performance ([EPIC-028](docs/issues/epics/EPIC-028-extensibility-plugin-modding-and-python-api.md))

Introduces out-of-process scripting and modding, first-class connectors for Google Gemini and DeepSeek, and high-frame-rate PDF rendering improvements.

### 5.1 Python Scripting API, Hooks, Cron & In-App Console ([EPIC-028](docs/issues/epics/EPIC-028-extensibility-plugin-modding-and-python-api.md))
- **Out-of-Process JSON-RPC Python API:** Allows users to write Python scripts and plugins that query and update vault items, metadata, and schedules over `stdin`/`stdout` JSON-RPC without recompiling Rust code or violating StudyFrog's `unsafe_code = "forbid"` rule.
- **Three Execution Modes:**
  1. **Cron Jobs:** Scheduled periodic tasks (such as automated vault health checks or scheduled exports).
  2. **Event-Bound Hooks:** Scripts triggered by lifecycle events (`on_item_saved`, `on_rehearsal_completed`, `on_cron_tick`).
  3. **Built-In Interactive Console:** An in-app console view for executing commands and ad-hoc scripts interactively.

### 5.2 Google Gemini Provider Integration ([STORY-0083](docs/issues/stories/STORY-0083-google-gemini-in-models-for-studyfrog.md))
- Adds `Protocol::Gemini` as a first-class provider in the Assistant settings alongside Ollama, OpenAI-compatible, and Anthropic.
- Supports Google's `v1beta/models/{model}:generateContent` wire schema (`gemini-1.5-flash`, `gemini-1.5-pro`, etc.), authenticates via the `x-goog-api-key` header (preventing key leakage in URLs), parses safety `finishReason` codes cleanly, and stores API keys in `secrets.toml` with `0600` permissions.

### 5.3 DeepSeek Provider Integration ([STORY-0084](docs/issues/stories/STORY-0084-deepseek-in-models-for-studyfrog.md))
- Adds `Protocol::Deepseek` as a dedicated provider preset (`https://api.deepseek.com`) with model presets (`deepseek-chat`, `deepseek-reasoner`), `Bearer` authentication stored in `secrets.toml` (`0600`), and support for reasoning responses during interactive exams and vault queries.

### 5.4 PDF Viewer Performance & High-DPI Sharpness ([STORY-0085](docs/issues/stories/STORY-0085-performance-optimization-in-studyfrog-pdf-viewer.md), [BUG-0010](docs/issues/bugs/BUG-0010-pdf-pages-are-illegible-depending-on-zoom-or-page-fit-mode.md))
- **60 FPS Scrolling & Bounded LRU Texture Cache (`STORY-0085`):** Virtualizes sidebar page thumbnail rendering so only visible thumbnails are queued, reuses `hayro::RenderCache` across page renders in a document session, segregates thumbnail caches from full-page caches, and bounds memory with an explicit LRU cache.
- **High-Resolution Zoom & Fit-Mode Sharpness (`BUG-0010`):** Prevents low-resolution sidebar thumbnail fallbacks (`250` milli-scale) from poisoning high-resolution page texture keys when switching between `Fit to Width`, `Fit Page`, and custom zoom levels.

---

## 6. Behavioral Learning Analytics, FSRS Optimizer & Workload Forecasting ([EPIC-034](docs/issues/epics/EPIC-034-behavioral-learning-analytics-fsrs-optimizer-and-workload-forecasting.md))

Expands `studyfrog-report` from basic interval-bucket retention and geometric review-count projections into a comprehensive behavioral telemetry and state-simulation forecasting suite.

### 6.1 Chronobiological (Hour-of-Day & Day-of-Week) Performance Telemetry ([STORY-0102](docs/issues/stories/STORY-0102-chronobiological-hour-of-day-and-day-of-week-performance-telemetry.md))
- Aggregates recall rate, `Good`/`Easy` share, review volume, and median/p90 answer latency across the 24 local hours of the day and 7 days of the week.
- Identifies and highlights the learner's biological **Peak Study Window** and circadian fatigue troughs in the Reports screen.

### 6.2 Within-Session Cognitive Fatigue & Pacing Decay Analysis ([STORY-0103](docs/issues/stories/STORY-0103-within-session-cognitive-fatigue-and-pacing-decay-analysis.md))
- Analyzes pause-adjusted `run_event` logs across elapsed session duration buckets (`0–5 min`, `5–10 min`, `10–15 min`, `15–20 min`, `20–30 min`, `30+ min`) and item sequence positions.
- Detects the **Fatigue Inflection Point** where response latency spikes or recall accuracy drops, recommending an optimal rehearsal session duration.

### 6.3 Leech Diagnostics, Interference Detection & Confusion Clusters ([STORY-0104](docs/issues/stories/STORY-0104-leech-diagnostics-interference-detection-and-confusion-clusters.md))
- **Multi-Signal Leech Detection:** Flags items using total lapses, consecutive `Hard`/`Again` streaks, cumulative review-time burden relative to the stack median, and stagnant stability-to-review ratios.
- **Interference & Confusion Pair Detection:** Identifies pairs of items in a stack (or connected by `[[Wikilinks]]`/shared tags and high token/embedding similarity) that repeatedly lapse in close temporal proximity, offering one-click actions to edit, pause, or launch a contrastive side-by-side drill.

### 6.4 FSRS State-Simulation Forecasting, Study-Minute Projection & Exam Pacer ([STORY-0105](docs/issues/stories/STORY-0105-fsrs-state-simulation-forecasting-study-minute-projection-and-exam-pacer.md))
- **State-Transition & Monte Carlo Workload Forecast:** Simulates each item's actual FSRS-6/SM-2 `MemState` forward over `30`, `90`, or `365` days with P10/P50/P90 confidence bands, adherence/postponement modeling, and stage breakdowns (`New`, `Learning/Relearning`, `Young`, `Mature`).
- **Study-Minute Projection:** Converts projected daily review counts into expected **study minutes** using empirical per-stack answer and reading latencies.
- **Exam Deadline Pacer:** Calculates the exact daily new-item and review quota required for ≥ 90% of a stack's items to reach target retrievability before a configured exam or due date.

### 6.5 Workload vs. Retention Simulator & Minimum Recommended Retention — CMRR ([STORY-0106](docs/issues/stories/STORY-0106-workload-versus-retention-simulator-and-minimum-recommended-retention.md))
- Sweeps candidate desired retention targets from `0.75` to `0.97` over a simulated 180–365 day horizon to plot **Daily Study Minutes vs. Desired Retention**.
- Computes the learner's **Minimum Recommended Retention (CMRR)**—the inflection point that maximizes retained knowledge per minute of study time without triggering a relearning avalanche—with a one-click apply action in Stack Settings.

### 6.6 Empirical Forgetting Curves, True Retention & Stability-Tier Survival Analysis ([STORY-0107](docs/issues/stories/STORY-0107-empirical-forgetting-curves-true-retention-and-stability-tier-survival-analysis.md))
- Computes **True Retention** (recall rate on mature items $S \ge 21\text{d}$ reviewed within $\pm 20\%$ of their scheduled interval) separately from **Young Retention** and **Overdue Retention**.
- Plots empirical forgetting curves grouped by stability tier (`<7d`, `7–30d`, `30–90d`, `90d+`) against FSRS theoretical retrievability $R(t, S)$.

---

## 7. Rich Active-Recall Formats, Inline Note Extraction & Focus Workflows ([EPIC-035](docs/issues/epics/EPIC-035-rich-active-recall-formats-inline-note-extraction-and-focus-workflows.md))

Bridges note-taking, visual diagram memorization, discriminative interleaving, and structured focus blocks.

### 7.1 Inline Cloze Deletions (`{{c1::answer::hint}}`) ([STORY-0108](docs/issues/stories/STORY-0108-inline-cloze-deletions-in-flashcards-and-notes.md))
- Supports `{{c1::answer}}` and `{{c1::answer::hint}}` syntax across multiple numbered cloze groups (`c1`, `c2`, ...) in flashcards and notes, with an editor formatting toolbar button and full support in `Flip cards`, `Write answers`, and `Exam` rehearsal modes.

### 7.2 Image Occlusion Cards for Diagrams, Slides & PDF Figures ([STORY-0109](docs/issues/stories/STORY-0109-image-occlusion-cards-for-diagrams-slides-and-pdf-figures.md))
- Interactive canvas to draw rectangular occlusion masks over any vault image or extracted PDF figure, storing normalized `[0.0, 1.0]` coordinates in plain-text Markdown without modifying the image file.
- Supports both *Hide All, Guess One* and *Hide One, Guess One* rehearsal modes.

### 7.3 Inline Flashcard & Question Extraction from Markdown Notes ([STORY-0110](docs/issues/stories/STORY-0110-inline-flashcard-and-question-extraction-from-markdown-notes.md))
- Recognizes inline `Concept :: Definition` descriptor lines and `Q: ... / A: ...` blocks inside Markdown notes and provides a one-click `Extract / Sync cards` action in the Note editor that idempotently creates/updates linked cards with bidirectional `[[Note Title#Heading]]` backlinks.

### 7.4 Contrastive Side-by-Side Drill for Confused Items ([STORY-0111](docs/issues/stories/STORY-0111-contrastive-side-by-side-drill-for-confused-and-interfering-items.md))
- Launches a side-by-side comparison drill from any detected `ConfusionPair` in Reports (`STORY-0104`) or multi-item selection in the Library, prompting the learner to discriminate between look-alike concepts and record a shared disambiguation note.

### 7.5 Pomodoro Focus Timer with Diary & Session Logging ([STORY-0112](docs/issues/stories/STORY-0112-pomodoro-focus-timer-with-automatic-learning-diary-and-session-logging.md))
- Adds a compact top-bar Focus / Pomodoro timer (`25/5`, `50/10`, or custom work/break intervals) that runs across PDF reading, note writing, and rehearsal, attributes focused minutes to the active stack, and offers a one-click Learning Diary reflection when a work block finishes.

---

## 8. Evidence-Based UI Aesthetics, Perceptual Color Science & UX Refinement ([EPIC-036](docs/issues/epics/EPIC-036-evidence-based-ui-aesthetics-perceptual-color-science-and-ux-refinement.md))

Grounded in empirical HCI and psychophysical research ([docs/research/ui-aesthetics-and-ux-science.md](docs/research/ui-aesthetics-and-ux-science.md)), this epic refines StudyFrog's visual hierarchy, color perception, typography, and cognitive load.

### 8.1 Perceptual OKLCH Palette Calibration & WCAG 3 APCA Verification ([STORY-0113](docs/issues/stories/STORY-0113-perceptual-oklch-color-palette-calibration-and-apca-contrast-verification.md))
- Calibrates all 6 accents (`Blue`, `Green`, `Purple`, `Orange`, `Pink`, `Teal`), semantic statuses, and surface elevation steps in **OKLCH** perceptual color space so every hue has matched perceived lightness ($L$) and chroma ($C$).
- Enforces automated **WCAG 3 APCA ($L_c$)** perceptual contrast tests ($|L_c| \ge 75$ for body text, $|L_c| \ge 58$ for secondary labels, and $|L_c| \in [15, 28]$ for subtle hairlines).

### 8.2 Psychophysical Typographic Measure, Modular Scale & Tabular Numerals ([STORY-0114](docs/issues/stories/STORY-0114-psychophysical-typographic-measure-modular-hierarchy-and-tabular-numerals.md))
- Constrains long-form reading, note preview, and rehearsal card columns to the empirically optimal **55–75 characters per line** (`620–680 px` at `100%` scale) to prevent return-sweep eye fatigue.
- Standardizes a 4-tier modular typographic hierarchy and tabular digit alignment so ticking timers and numeric columns never jitter horizontally.

### 8.3 Low-Entropy Surface Elevation, Border De-Cluttering & Gestalt Spacing ([STORY-0115](docs/issues/stories/STORY-0115-low-entropy-surface-elevation-border-decluttering-and-gestalt-spacing.md))
- Replaces nested 1 px box-in-box borders across Reports, Home, Stack workspace, and Details tabs with stepped OKLCH surface elevation (`background` -> `panel` -> `surface` -> `surface_hover`) and strict `4 / 8 / 16 / 24 px` Gestalt proximity spacing.
- Enforces Hick's Law and the Signaling Principle with at most one high-salience primary CTA button per screen.

### 8.4 Distraction-Free Zen Focus Mode & Zero-Layout-Shift Polish ([STORY-0116](docs/issues/stories/STORY-0116-distraction-free-zen-focus-mode-and-zero-layout-shift-interaction-polish.md))
- Adds a toggleable (and optional auto-rehearsal) **Zen Focus Mode** (`Ctrl+Shift+F`) that hides the left sidebar and status bar chrome, paired with Fitts's Law target sizing ($\ge 36\text{ px}$ rating buttons) and zero vertical layout shift when flipping cards.

---

## 9. State-of-the-Art Model Context Protocol (MCP) Server, Client Host & Registry ([EPIC-037](docs/issues/epics/EPIC-037-state-of-the-art-model-context-protocol-mcp-server-client-and-registry.md))

A future-proof, data-oriented MCP implementation ([docs/research/mcp-architecture.md](docs/research/mcp-architecture.md)) allowing StudyFrog to act as both an **MCP Server** (`studyfrog-app --mcp`) and an **MCP Host/Client**.

### 9.1 Pure JSON-RPC 2.0 MCP Engine & Plain-Data Registry ([STORY-0117](docs/issues/stories/STORY-0117-pure-json-rpc-2-0-mcp-protocol-engine-and-plain-data-registry.md))
- Transport-agnostic JSON-RPC 2.0 MCP router with `McpRegistry` storing `McpTool` (with `ToolAnnotations`: `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`), `McpResourceTemplate`, and `McpPrompt` descriptors as plain data structs of function pointers (zero OOP traits, 100% P1 compliant).

### 9.2 MCP Server Resources (`studyfrog://`) & Read-Only Search/Analytics Tools ([STORY-0118](docs/issues/stories/STORY-0118-studyfrog-mcp-server-resources-and-read-only-search-and-analytics-tools.md))
- Exposes `studyfrog://vault/summary`, `studyfrog://stacks/{stack}`, `studyfrog://items/{id_or_key}`, `studyfrog://reports/{kind}`, and `studyfrog://diary/{date}` resources, plus read-only tools (`vault_search_items`, `vault_get_due_queue`, `analytics_get_report`, `forecast_simulate_workload`, `pdf_read_pages`).

### 9.3 Annotated Authoring Tools, Human-in-the-Loop Staging & MCP Prompts ([STORY-0119](docs/issues/stories/STORY-0119-annotated-mcp-authoring-tools-human-in-the-loop-staging-and-prompts.md))
- Exposes all built-in StudyFrog academic prompts via `prompts/list` and `prompts/get`, plus staged authoring tools (`vault_propose_items`, `note_extract_inline_cards`, `diary_log_entry`, `calendar_plan_session`) and policy-gated structural tools (`vault_move_stack`, `vault_delete_item` with undo).

### 9.4 CLI `stdio` MCP Server & Localhost Streamable HTTP Transport ([STORY-0120](docs/issues/stories/STORY-0120-cli-stdio-mcp-server-mode-and-localhost-streamable-http-transport.md))
- Adds `studyfrog-app --mcp [--vault <path>] [--mcp-allow-write]` for standard `stdin`/`stdout` connections (Claude Desktop, Cursor, Zed, Cline), an optional `127.0.0.1` Streamable HTTP listener with bearer token and `Origin` checks, and a one-click config snippet generator in the UI.

### 9.5 Built-In Assistant MCP Client Host ([STORY-0121](docs/issues/stories/STORY-0121-built-in-assistant-mcp-client-host-for-external-academic-tool-servers.md))
- Enables StudyFrog's built-in Assistant to connect to external MCP tool servers (such as Zotero citation servers, arXiv/PubMed search, or local computation tools) with capability discovery and human-in-the-loop approval for non-read-only tool calls.

---

## 10. CLI Control Protocol for Interactive AI Agent Operation & `~/.local/` App Files ([EPIC-038](docs/issues/epics/EPIC-038-cli-control-protocol-for-interactive-ai-agent-application-operation.md))

Provides AI agents and power users with interactive CLI control over every StudyFrog screen, action, and state transition, backed by a standardized `~/.local/share/studyfrog/` directory for configurations, runtime sockets, and rotating console logs.

### 10.1 Standardized `~/.local/` Application Files, Logs & Configurations ([STORY-0122](docs/issues/stories/STORY-0122-studyfrog-application-files-in-local-with-logs-and-configurations.md))
- Consolidates global configurations (`config/app.toml`, `config/secrets.toml`), user profiles (`profiles/`), downloaded local models (`models/`), runtime sockets (`run/control.sock`), and rotating logs (`logs/studyfrog.log` and `logs/console.jsonl`) under `~/.local/share/studyfrog/`.
- Logs every UI notice, service event, worker diagnostic, and error to `logs/console.jsonl` so CLI users and AI agents can inspect or tail application logs (`studyfrog-app logs --follow`).

### 10.2 Interactive CLI Control Protocol & Console Log Streaming ([STORY-0124](docs/issues/stories/STORY-0124-interactive-cli-control-protocol-and-console-log-streaming-for-ai-agents.md))
- Allows AI agents to interactively operate a live or headless StudyFrog instance via `studyfrog-app ctl <command>` (over `~/.local/share/studyfrog/run/control.sock`) or `studyfrog-app --control-stdio`.
- Grants full access to all application features: screen navigation, live `State` and semantic widget tree inspection, item/stack CRUD, stepping through rehearsal and exam runs (`start`, `flip`, `answer`, `rate`, `mark`, `pause`, `stop`), query execution, PNG screenshots, and real-time console log streaming.



