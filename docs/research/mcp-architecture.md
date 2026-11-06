# State-of-the-Art Model Context Protocol (MCP) Architecture for StudyFrog

Architectural analysis and specification for integrating the **Model Context Protocol (MCP)** into StudyFrog in a future-proof, extensible, and safety-first manner that respects all workspace guardrails ([DESIGN_PRINCIPLES](../DESIGN_PRINCIPLES.md): P1 zero inherent `impl` blocks, P2 ≤ 250 lines per file, P3 strict crate layering, P9 `unsafe_code = "forbid"`).

---

## 1. Strategic Role of MCP in StudyFrog

The **Model Context Protocol (MCP)** (Anthropic / Linux Foundation Agentic AI Foundation, JSON-RPC 2.0 specification) is the industry standard connecting AI hosts, IDEs, and autonomous agents with structured domain tools, contextual resources, and reusable prompt templates.

Supporting MCP in StudyFrog unlocks **two symmetrical capabilities**:

1. **StudyFrog as an MCP Server (`studyfrog-app --mcp`):**
   - Any external MCP client (Claude Desktop, Cursor, Zed, Windsurf, Cline, Goose, or custom research pipelines) can connect to a StudyFrog vault via standard `stdio` or local Streamable HTTP.
   - Instead of external LLMs blindly editing raw `.md` files (risking broken YAML front-matter, missing ULIDs, or out-of-sync SQLite indices), StudyFrog exposes **typed Resources**, **annotated Tools**, and **parameterized Prompts** backed directly by `studyfrog-service`.
2. **StudyFrog as an MCP Host / Client (Built-In Assistant):**
   - StudyFrog's built-in Assistant (`studyfrog-agent`) can connect to user-configured external MCP servers (e.g., Zotero/BibTeX citation managers, PubMed/arXiv literature search, local Python/Rust academic calculators) to enrich note authoring, concept mapping, and interactive examinations.

---

## 2. State-of-the-Art Protocol Design (2025/2026 MCP Specification)

A future-proof MCP implementation must separate **protocol framing**, **capability discovery**, **plain-data handler registration**, and **physical transport**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         External MCP Clients / Hosts                    │
│              (Claude Desktop, Zed, Cursor, Custom Agents)               │
└───────────────────┬─────────────────────────────────┬───────────────────┘
                    │ stdio (stdin/stdout JSON-RPC)   │ Streamable HTTP (localhost)
┌───────────────────▼─────────────────────────────────▼───────────────────┐
│              Transport Adapters (`studyfrog-app` / `studyfrog-net`)     │
├─────────────────────────────────────────────────────────────────────────┤
│         Pure MCP Protocol & Router Engine (`studyfrog-mcp`)             │
│   • JSON-RPC 2.0 Envelope, Batching, ID Correlation, Error Codes        │
│   • Capability Negotiation (`initialize`, `ping`, stateless context)    │
│   • Plain-Data Registry (`Vec<McpTool>`, `Vec<McpResource>`, `Prompts`) │
│   • Safety Gate (`ToolAnnotations`: readOnly / destructive / staged)    │
├─────────────────────────────────────────────────────────────────────────┤
│                  Application Service Layer (`studyfrog-service`)        │
│     (VaultSession, Query, Rehearsal, Reports, Forecast, Prompts, PDF)   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Why a Plain-Data Registry (`studyfrog-ext` Pattern) Is Future-Proof
Many Rust MCP SDKs rely heavily on deeply nested async trait hierarchies and procedural macros, which conflict with StudyFrog's **Rule P1** (zero inherent `impl` blocks) and **Rule P2** (≤ 250 lines per file) and couple business logic to specific async runtimes.

Instead, StudyFrog applies the exact data-oriented pattern proven in `studyfrog-ext`:
- Every **MCP Tool**, **MCP Resource Template**, and **MCP Prompt** is a **plain data struct** containing static metadata, a JSON Schema (`2020-12`) value, MCP `ToolAnnotations`, and a **pure function pointer**:

```rust
pub struct McpToolAnnotations {
    pub title: &'static str,
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

pub struct McpTool {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: &'static str,   // JSON Schema 2020-12
    pub output_schema: Option<&'static str>,
    pub annotations: McpToolAnnotations,
    pub call: fn(&mut VaultSession, &str) -> Result<McpToolResult, McpError>,
}
```

- **Zero-Friction Extensibility:** Adding a new tool, resource, or prompt—whether in core StudyFrog or from an external extension crate—is a single slice entry in `McpRegistry`. No protocol routing code ever changes when new capabilities are added.

---

## 3. Canonical Primitives Exposed by StudyFrog MCP Server

### 3.1 Resources (`resources/list`, `resources/templates/list`, `resources/read`)
All vault entities are addressable via canonical `studyfrog://` URIs returning structured JSON or clean Markdown with MIME types (`application/json` or `text/markdown`):

| URI Pattern | MIME Type | Description |
| :--- | :--- | :--- |
| `studyfrog://vault/summary` | `application/json` | Vault metadata, stack hierarchy tree, total item counts, due counts, and active streak. |
| `studyfrog://stacks/{stack}` | `application/json` | Stack settings (scheduler, desired retention, exam date), familiarity score, and child items. |
| `studyfrog://items/{id_or_key}` | `text/markdown` | Full Markdown document (front-matter + body) and per-stack FSRS/SM-2 scheduling state. |
| `studyfrog://reports/retention` | `application/json` | Retention buckets, True Retention, stability-tier survival curves, and replay verdict. |
| `studyfrog://reports/forecast` | `application/json` | 30/90-day review and study-minute forecast, backtest error, CMRR curve, and Exam Pacer. |
| `studyfrog://reports/behavior` | `application/json` | Chronobiological 24h/7d telemetry, within-session fatigue curve, and leech/interference pairs. |
| `studyfrog://diary/{date}` | `text/markdown` | Learning Diary entries and self-assessment scale ratings for a date (`YYYY-MM-DD`). |

### 3.2 Tools (`tools/list`, `tools/call`) with Safety Annotations
Tools are partitioned into three explicit safety tiers via MCP `ToolAnnotations` and capability scopes:

1. **Tier 1 — Read-Only Query & Analytics Tools (`readOnlyHint: true`, `idempotentHint: true`, Always Enabled):**
   - `vault_search_items`: Run structured StudyFrog queries (`kind:card tag:exam priority:high ORDER BY due ASC`) or full-text search across items and indexed PDFs.
   - `vault_get_due_queue`: Retrieve the prioritized rehearsal queue for a stack or vault (`DueFirst`, `LowestRecall`, etc.).
   - `analytics_get_report`: Retrieve retention, latency, chronotype, session fatigue, leech/interference pairs, or self-assessment calibration reports.
   - `forecast_simulate_workload`: Run forward FSRS workload simulations, Exam Deadline Pacer calculations, or CMRR retention sweeps.
   - `pdf_read_pages`: Extract text and existing highlights/annotations from a vault PDF by page range.
2. **Tier 2 — Staged / Additive Authoring Tools (`readOnlyHint: false`, `destructiveHint: false`):**
   - `vault_propose_items`: Stage new or updated flashcards, questions, cloze items, or notes into StudyFrog's human-in-the-loop approval queue (or write directly when `--mcp-write` is enabled), validating front-matter and repairing CSV/Markdown formatting automatically.
   - `note_extract_inline_cards`: Parse a vault note for `Concept :: Definition` or `Q: / A:` blocks and preview/sync linked cards.
   - `diary_log_entry`: Create or append a structured Learning Diary reflection with 1–5 scale ratings.
   - `calendar_plan_session`: Schedule a study session on the vault calendar.
3. **Tier 3 — Structural / Destructive Tools (`destructiveHint: true`, Disabled unless `--mcp-allow-destructive` or Interactive Elicitation Approval):**
   - `vault_move_stack`: Move a stack under a parent stack or promote to standalone top-level while preserving item schedules.
   - `vault_delete_item`: Delete an item (recording a full undo entry in the session undo stack).

### 3.3 Prompts (`prompts/list`, `prompts/get`)
All built-in StudyFrog academic prompts (`flashcards`, `questions`, `stack`, `notes`, `summary`, `critique`, `explain`, `exam`, `socratic`, `concept_map`) are automatically exposed via `prompts/list` and `prompts/get` with typed arguments (`file_path`, `level`, `question_count`), allowing external MCP clients to invoke StudyFrog's importer-tested prompts natively.

---

## 4. Transport, Security, and Crate Layering

1. **Crate Placement (`scripts/check-deps.sh`):**
   - `studyfrog-mcp` (or `studyfrog-agent::mcp`): Pure JSON-RPC 2.0 parser, serializer, capability negotiator, and `McpRegistry` types. Zero I/O, zero network dependencies, 100% unit-testable with golden JSON-RPC fixtures.
   - `studyfrog-service::mcp_handlers`: Pure domain bridges mapping `McpRegistry` tool/resource calls to `VaultSession` operations.
   - `studyfrog-app`: CLI flag `--mcp [--vault <path>] [--allow-write]` running a clean `stdin`/`stdout` JSON-RPC loop (redirecting all diagnostic logs to `stderr` as mandated by the MCP `stdio` specification).
   - `studyfrog-net`: Optional localhost-bound (`127.0.0.1`) Streamable HTTP transport with bearer token authentication and Origin validation to prevent DNS-rebinding or cross-origin attacks.
2. **Human-in-the-Loop & Elicitation Safety:**
   - By default, mutation tool calls from external MCP clients can either target the **Staged Proposals Queue** (visible in the StudyFrog UI for 1-click review/accept) or request interactive user confirmation via MCP `elicitation/create`.
