# M12: External models, PDF viewer fixes and optimization, and Python modding API (0.5.0)

> **Status after 0.5.0 (2026-11-08):** released with part of this scope. Built: STORY-0083, STORY-0084, BUG-0010,
> STORY-0086, STORY-0087, STORY-0088 to STORY-0092, TASK-0233, STORY-0098, STORY-0117 and STORY-0122; partly built:
> STORY-0085, STORY-0118 and STORY-0120. Not built and moved on: EPIC-033, STORY-0096, STORY-0097, STORY-0123,
> STORY-0102 to STORY-0107, STORY-0108 to STORY-0112, STORY-0113 to STORY-0116, STORY-0119, STORY-0121, STORY-0124 and
> the Python modding API. The details are in `priority-list.md` and `docs/RELEASE_NOTES_0.5.0.md`.

**Goal:** Connect external intelligence models (Google Gemini and Deepseek) via native APIs, fix PDF illegibility defects and optimize rendering performance to 60 fps, and expose a Python modding API for plugins, cron jobs, event handlers, and an in-app console.

**Entry criteria:** 0.4.0 released (M11 exit met).

## Epics, stories, bugs

- **EPIC-028** Extensibility, plugin modding, and Python API
  - **STORY-0083** Google Gemini in models for StudyFrog
    - Gemini Generative Language API integration (`v1beta/models/{model}:generateContent`)
    - Role formatting (`user` and `model`) and system instruction handling
    - Secure API key management via `secrets.toml` with `0600` permissions and `x-goog-api-key` header
    - Assistant Settings UI provider option and connection test
  - **STORY-0084** Deepseek in models for StudyFrog
    - Dedicated `Protocol::Deepseek` with base URL `https://api.deepseek.com`
    - Support for `deepseek-chat` and `deepseek-reasoner` models
    - Secure Bearer token authentication and error handling
    - Assistant Settings UI provider option and connection test
  - **STORY-0085** Performance optimization in StudyFrog pdf-viewer
    - Virtualized thumbnail rendering in sidebar to eliminate unneeded page renders
    - Persistent `hayro::RenderCache` reuse across page renders in an active document session
    - Segregated thumbnail and high-resolution picture caches to prevent eviction thrashing
    - Bound memory consumption with explicit LRU texture cache maintaining 60 fps
  - **BUG-0010** PDF pages are illegible depending on zoom or page fit mode
    - Decouple requested render resolution key from cached fallback thumbnail picture key
    - Invalidate or replace texture cache handles when higher-resolution raster pixels arrive
    - Ensure crisp rasterization across FitWidth, FitPage, and all zoom levels
- **EPIC-029** Expanded achievements and per-stack progression
  - **STORY-0086** Expanded achievement catalog and milestone tiers
    - Progressive milestone tiers for streaks, finished runs, ratings, and mastered items
    - Graduated per-stack goals for learned cards, deck mastery, and practice volume
    - Preserve backward compatibility with historical unlock records
  - **STORY-0087** Per-stack achievement UI and filtering
    - Filtering in the achievements screen (All, Global, or specific Stack)
    - Stack browser achievement progress pills and summary badges
    - Grouped/badged layout for clean readability across multiple stacks
- **EPIC-030** Prompt expansion
  - **STORY-0088** Multi-level file explanation prompt
    - Selectable explanation depth: Intuitive Analogy (ELI5), Core Concepts, and Technical Deep-Dive
    - Clear structured Markdown notes with tags ready for vault import
  - **STORY-0089** Interactive file examination and feedback prompt
    - Interactive oral/written diagnostic exam based on an attached document
    - Evaluation rubric covering accuracy, omissions, and model answer elaboration
    - Follow-up flashcard and question recommendations targeting identified gaps
  - **STORY-0090** Socratic dialogue and active inquiry prompt
    - Step-by-step questioning model leading the learner to discover core conclusions
    - Misconception detection with counter-examples and guided hints
    - Concluding dialogue synthesis note exportable to vault
  - **STORY-0091** Concept mapping and prerequisite analysis prompt
    - Prerequisite knowledge dependency chains extracted from dense texts
    - Conceptual contrast tables distinguishing easily conflated terms
    - Markdown notes formatted with wikilinks for the vault graph
  - **STORY-0092** Prompt library categorization and parameterized execution UI
    - Grouped library categories: Content Generation, Interactive Tutoring & Exam, and Analysis
    - Interactive parameter controls in the Assistant Prompts UI
    - Search and workflow filtering
    - TASK-0233 Move prompt library to standalone left sidebar navigation
- **EPIC-032** Item keys, colorization, and reporting export
  - **STORY-0096** Item keys for querying
    - Sequential human-readable item keys (STACK_1, FLASHCARD_500, NOTE_12, QUESTION_3)
    - Query engine and search bar filtering and direct jump by key
    - Display key badges across details notebook tabs, editor header, and browser
  - **STORY-0097** Item colorization with color picker and viewer background styling
    - Item frontmatter color field supporting hex codes and curated pastel palette tokens
    - Color picker widget in the editor and details notebook tabs
    - Accessible tinted background rendering in the rehearsal card viewer and browser
  - **STORY-0098** Reporting export as CSV
    - Prominent Export as CSV action button in the Reports screen
    - Structured CSV tables covering review volume, retention rates, mastery, and run logs
    - Native file save dialog and vault exports location support
  - **STORY-0123** Colorization field for items influencing details and rehearsal view background
    - Colorization field in item front-matter and Details Properties tab
    - Theme-aware background tinting in the Item Details view, Editor, and Rehearsal card surface
- **EPIC-033** Exam run mode with mandatory text entry, semantic comparison, and percentage grading
  - Strict one-way exam progression (no skipping, no moving back, mandatory written submission)
  - Automatic semantic comparison scoring for flashcards and questions
  - Percentage grading (`60%` pass threshold `D`/`4.0` and `5%` intervals up to `95%–100%` `A+`/`1+`)
- **EPIC-034** Behavioral learning analytics, FSRS optimizer, and workload forecasting
  - **STORY-0102** Chronobiological hour-of-day and day-of-week performance telemetry
  - **STORY-0103** Within-session cognitive fatigue and pacing decay analysis
  - **STORY-0104** Leech diagnostics, interference detection, and confusion clusters
  - **STORY-0105** FSRS state-simulation forecasting, study-minute projection, and exam deadline pacer
  - **STORY-0106** Workload-versus-retention simulator and minimum recommended retention (CMRR)
  - **STORY-0107** Empirical forgetting curves, true retention, and stability-tier survival analysis
- **EPIC-035** Rich active-recall formats, inline note extraction, and focus workflows
  - **STORY-0108** Inline cloze deletions in flashcards and notes
  - **STORY-0109** Image occlusion cards for diagrams, slides, and PDF figures
  - **STORY-0110** Inline flashcard and question extraction from Markdown notes
  - **STORY-0111** Contrastive side-by-side drill for confused and interfering items
  - **STORY-0112** Pomodoro focus timer with automatic learning diary and session logging
- **EPIC-036** Evidence-based UI aesthetics, perceptual color science, and cognitive-load UX refinement
  - **STORY-0113** Perceptual OKLCH color palette calibration and WCAG 3 APCA contrast verification
  - **STORY-0114** Psychophysical typographic measure, modular hierarchy, and tabular numerals
  - **STORY-0115** Low-entropy surface elevation, border de-cluttering, and Gestalt proximity spacing
  - **STORY-0116** Distraction-free Zen focus mode and zero-layout-shift interaction polish
- **EPIC-037** State-of-the-art Model Context Protocol (MCP) server, client host, and extensible registry
  - **STORY-0117** Pure JSON-RPC 2.0 MCP protocol engine and extensible plain-data capability registry
  - **STORY-0118** StudyFrog MCP server resources and read-only search, analytics, and forecast tools
  - **STORY-0119** Annotated MCP authoring tools, human-in-the-loop staging, and parameterized MCP prompts
  - **STORY-0120** CLI stdio MCP server mode and localhost Streamable HTTP transport
  - **STORY-0121** Built-in Assistant MCP client host for external academic tool servers
- **EPIC-038** CLI control protocol for interactive AI agent application operation
  - **STORY-0122** StudyFrog application files in `~/.local/` with logs and configurations
  - **STORY-0124** Interactive CLI control protocol and console log streaming for AI agents

## Priority order

1. **Defect fix first:** BUG-0010 (PDF pages illegible under zoom/page fit) to eliminate text rendering degradation for all PDF reading workflows.
2. **PDF performance optimization:** STORY-0085 (virtualized thumbnail rendering, RenderCache reuse, and LRU texture caching) to ensure 60 fps navigation.
3. **Model integrations:** STORY-0083 (Google Gemini) and STORY-0084 (Deepseek) expanding external model connectivity for vault assistance and exams.
4. **Extensibility & modding:** EPIC-028 Python API, subprocess JSON-RPC bridge, cron scheduler, event triggers, and built-in interactive console.
5. **Achievements expansion:** EPIC-029 (STORY-0086, STORY-0087) tiered achievement catalog, per-stack progression milestones, and achievements UI filtering.
6. **Prompt expansion:** EPIC-030 (STORY-0088 to STORY-0092) multi-level explanation, oral exams, Socratic dialogue, concept mapping, and categorized prompt UI.
7. **Vault organization & insights:** EPIC-032 (STORY-0096 to STORY-0098) item keys for querying, item colorization, and reporting CSV export.

**Exit criteria:**
- External model API integration for Google Gemini and Deepseek functioning in the Assistant and Rehearsal Exam screens
- PDF viewer pages remain crisp and legible across all zoom settings and fit modes; rendering performance meets 60 fps target
- Python API exposed for StudyFrog, enabling modding plugins and automation via cron triggers, event listeners, or in-app console
- Tiered default achievements and per-stack filtering functioning in the achievements screen and stack browser
- Expanded prompt library with categorized navigation, parameterized controls, multi-level explanations, interactive exams, Socratic dialogues, and concept mapping templates functional in the Assistant
- Item keys functional in query search, color picker and background tinting working across editor and rehearsal, and reporting metrics exportable as CSV
