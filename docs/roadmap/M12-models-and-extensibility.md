# M12: External models, PDF viewer fixes and optimization, and Python modding API (0.5.0)

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
  - **BUG-0011** Importer only accepts TSV format and fails on CSV prompt outputs
    - Auto-detect CSV vs TSV delimiters
    - Seamlessly import multi-column prompt outputs (stack,kind,title,front,back,tags)
    - Handle quoted strings, linebreaks, and semicolon-delimited tags cleanly

## Priority order

1. **Defect fix first:** BUG-0010 (PDF pages illegible under zoom/page fit) to eliminate text rendering degradation for all PDF reading workflows.
2. **PDF performance optimization:** STORY-0085 (virtualized thumbnail rendering, RenderCache reuse, and LRU texture caching) to ensure 60 fps navigation.
3. **Model integrations:** STORY-0083 (Google Gemini) and STORY-0084 (Deepseek) expanding external model connectivity for vault assistance and exams.
4. **Extensibility & modding:** EPIC-028 Python API, subprocess JSON-RPC bridge, cron scheduler, event triggers, and built-in interactive console.
5. **Achievements expansion:** EPIC-029 (STORY-0086, STORY-0087) tiered achievement catalog, per-stack progression milestones, and achievements UI filtering.
6. **Prompt expansion:** EPIC-030 (STORY-0088 to STORY-0092) multi-level explanation, oral exams, Socratic dialogue, concept mapping, and categorized prompt UI.

**Exit criteria:**
- External model API integration for Google Gemini and Deepseek functioning in the Assistant and Rehearsal Exam screens
- PDF viewer pages remain crisp and legible across all zoom settings and fit modes; rendering performance meets 60 fps target
- Python API exposed for StudyFrog, enabling modding plugins and automation via cron triggers, event listeners, or in-app console
- Tiered default achievements and per-stack filtering functioning in the achievements screen and stack browser
- Expanded prompt library with categorized navigation, parameterized controls, multi-level explanations, interactive exams, Socratic dialogues, and concept mapping templates functional in the Assistant
