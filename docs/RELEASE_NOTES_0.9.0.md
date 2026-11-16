# StudyFrog 0.9.0

FrogCanvas 2D spatial concept board, PDF precision area cropping with 1-click split navigation, continuous priority queue and anti-paralysis budget slicing, gamified multi-modal recall match mode with semantic distractor exams, native FSRS parameter optimization engine with inline card capture, and private offline local SLM assistant sidecar. The full list of changes is in `CHANGELOG.md` under `[0.9.0]`.

## What is new

- **FrogCanvas 2D Spatial Mind-Map & Concept Board (`crates/studyfrog-graph`, `crates/studyfrog-ui`).**
  - Infinite 2D spatial learning board where cards, notes, text labels, and visual excerpts can be arranged, clustered, and connected (EPIC-050).
  - TOML document serialization and data model (`_canvas.toml` in stack directories) for `CanvasDoc`, `CanvasNode`, `CanvasEdge`, and `CanvasFrame` (TASK-0253, STORY-0153).
  - 2D spatial quadtree index with recursive quadrant subdivision (NW, NE, SW, SE) and sub-millisecond bounding box viewport culling maintaining a steady 60 FPS during pan and zoom (TASK-0254, STORY-0153).
  - Interactive canvas view with smooth 2D affine matrix panning, zooming clamped between 0.05x and 20.0x, and interactive node dragging (TASK-0255, STORY-0154).
  - Directed and bidirectional cubic Bézier curve edge rendering with relationship labels, geometric connection anchors, and card preview popovers upon node selection or hover (TASK-0256, STORY-0154).
- **PDF Precision Area Cropping & 1-Click Split Navigation (`crates/studyfrog-pdf`, `crates/studyfrog-ui`).**
  - Rectangular drag-selection lasso tool in the PDF viewer to crop visual diagrams and equations directly into flashcards and notes (TASK-0257, STORY-0155, EPIC-051).
  - Frontmatter coordinate anchor URI schema (`source: path/to/doc.pdf#page=N&bbox=x,y,w,h`) linking visual excerpts to precise document coordinates (TASK-0257, STORY-0155).
  - High-performance bounded LRU memory and disk rasterization cache (`CropLruCache`) for fast card image rendering without full-page re-renders (TASK-0258, STORY-0155).
  - 1-click split-screen "PDF Jump" contextual navigation button rendered on rehearsal cards and notes containing coordinate anchors (TASK-0259, STORY-0156).
  - Synchronized split-pane PDF viewer positioning to target page with flashing rectangular highlight overlay on coordinates and adjustable split ratio from 0.2 to 0.8 (TASK-0260, STORY-0156).
- **Continuous Priority Queue & Anti-Paralysis Slicing (`crates/studyfrog-srs`, `crates/studyfrog-service`).**
  - Effective priority scoring function blending normalized user priority (40%), retrievability shortfall `1 - R(t)` (35%), and familiarity shortfall `1 - F(c)` (25%) into an urgency score in `[0.0, 1.0]` (TASK-0261, STORY-0157, EPIC-052).
  - Priority-ordered queue builder slicing daily overdue queues to user-configured session budgets (e.g. 30 cards), guaranteeing high-priority card review and preventing cognitive backlog paralysis (TASK-0262, STORY-0157, EPIC-052).
- **Gamified & Multi-Modal Active Recall: Match Mode & Distractor Exams (`crates/studyfrog-rehearsal`, `crates/studyfrog-candle`, `crates/studyfrog-service`, `crates/studyfrog-ui`).**
  - Interactive Match Mode: 3x4 tile pairing drills (6 pairs / 12 tiles) with PRNG shuffling, monotonic millisecond timing, mismatch time penalties, and consecutive match streak multipliers (TASK-0263, STORY-0158, EPIC-053).
  - Responsive match grid UI in egui with tile selection highlighting, match/mismatch visual feedback animations, and score summary reports (TASK-0264, STORY-0158, EPIC-053).
  - Semantic distractor generation: extracts plausible multiple-choice distractors from active stack/vault card answers using MiniLM embedding cosine nearest neighbors rather than random shuffling (TASK-0265, STORY-0159, EPIC-053).
  - Distractor exam view: interactive multiple-choice question exam drills with automated grading, instant feedback, and integration into academic grading bands (TASK-0266, STORY-0159, EPIC-053).
- **Algorithmic Optimization & Frictionless Capture: FSRS Auto-Tuning & Inline Cards (`crates/studyfrog-srs`, `crates/studyfrog-md`, `crates/studyfrog-store`, `crates/studyfrog-service`, `crates/studyfrog-ui`).**
  - Local Adam gradient descent optimizer tuning all 21 FSRS-6 parameters on SQLite historical review logs via binary cross-entropy log-loss minimization and L2 regularization shrinking toward published priors (TASK-0267, STORY-0160, EPIC-054).
  - Rehearsal Settings UI dashboard with "Optimize Parameters" action, RMSE evaluation metrics, and log-loss improvement tracking before applying weights (TASK-0268, STORY-0160, EPIC-054).
  - Inline card delimiter scanner parsing `Term :: Definition` (forward) and `Term ::: Definition` (reversible) pairs from markdown note bodies while cleanly ignoring colons in code fences, backticks, math blocks, and tables (TASK-0269, STORY-0161, EPIC-054).
  - SQLite virtual item indexing syncing inline cards directly into the vault index with line coordinate anchors (`source: note#L<line>`) without cluttering filesystem directories (TASK-0270, STORY-0161, EPIC-054).
- **Offline Local SLM Internal Assistant (`crates/studyfrog-agent`, `crates/studyfrog-net`, `crates/studyfrog-store`, `crates/studyfrog-service`, `crates/studyfrog-ui`).**
  - Managed background sidecar supervisor running local `llama-server` bound strictly to `127.0.0.1` on ephemeral ports (18000..=19000) with stdin watchdog termination preventing zombie processes (TASK-0271, STORY-0162, EPIC-055).
  - Automatic 5-minute idle shutdown supervisor stopping child processes after 300s of inactivity to achieve 100% RAM reclamation (0 MB idle footprint) with seamless on-demand reactivation (TASK-0272, STORY-0162, EPIC-055).
  - Curated GGUF model catalog featuring `Qwen2.5-1.5B-Instruct` (default, Apache 2.0), `Llama-3.2-1B-Instruct`, and `SmolLM2-360M-Instruct`, with chunked range-resuming downloader and SHA-256 integrity verification (TASK-0273, STORY-0163, EPIC-055).
  - Server-Sent Events (SSE) streaming client and real-time chat UI with pulsing "Thinking / Analyzing vault..." indicator during prefill transitioning into live token streaming (TASK-0274, STORY-0163, EPIC-055).
  - Local SQLite `vault_embeddings` table storing 384-dimensional dense vectors with sub-15ms cosine similarity ranking over note chunk embeddings for on-device RAG (TASK-0275, STORY-0164, EPIC-055).
  - Strict 1024-token prompt budgeting (system <= 150, history <= 150, context <= 768 tokens) limiting CPU prefill latency to under 3.5 seconds on consumer hardware (TASK-0276, STORY-0164, EPIC-055).
- **Multilingual Prompt Library & German Translations (`docs/prompts/de/*`, `crates/studyfrog-service`).**
  - Comprehensive German localized translations of the standard prompt library: Exam (`exam.md`), Flashcards (`flashcards.md`), Notes (`notes.md`), Questions (`questions.md`), Socratic dialogue (`socratic.md`), and Summary (`summary.md`).
  - Prompt language selector allowing toggling between English and German prompt templates.
- **Universal Transfer & Import Overhaul (`crates/studyfrog-ui`).**
  - Restructured modal transfer and import dialogs (`modal_transfer_import.rs`, `modal_transfer_preview.rs`, `modal_import.rs`) with multi-format detection, stack bulk operations, and live previews.

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
sha256sum -c StudyFrog-0.9.0-x86_64.AppImage.sha256
chmod +x StudyFrog-0.9.0-x86_64.AppImage
./StudyFrog-0.9.0-x86_64.AppImage
```

### Tarball

```sh
sha256sum -c StudyFrog-0.9.0-x86_64.tar.gz.sha256
tar -xzf StudyFrog-0.9.0-x86_64.tar.gz
cd StudyFrog-0.9.0
./bin/studyfrog
```

### Upgrading from 0.8.0

All existing vaults, notes, cards, and review histories remain fully compatible.
