---
id: STORY-0206
type: story
title: "Ollama setup assistant: hardware audit, live Ollama status and model suggestions that fit this machine"
status: review
epic: EPIC-055
parents: ['EPIC-055']
estimate: 5
labels: [m14, slm, ui, tooling]
acceptance_criteria:
  - "The assistant audits this machine on request and shows CPU (cores and instruction-set support that matters for inference), total and free memory, the GPU and its video memory where it can be read (NVIDIA, AMD, Apple), and free disk space in the models folder; a value that cannot be read is shown as unknown, never guessed"
  - "The assistant queries the local Ollama service (by default at 127.0.0.1:11434) for its version and the models already installed, and shows whether Ollama is missing, not running, running with no models, or ready"
  - "The assistant lists models suggested for this machine, ranked by fit: each suggestion shows its download size, the memory it needs, whether it fits in memory or video memory, whether it is already installed, and a short reason for its rank; the list updates when the audit or the installed models change"
  - "The assistant guides the setup step by step from the state it found (install Ollama, start the service, pull a model, run it), with each command shown in a copyable block and the next step highlighted; the steps are the same whether the user is on Linux, macOS or Windows"
  - "The user picks a suggested or installed model as StudyFrog's local model, and the assistant checks it with one short test prompt before it is used for scoring or grading"
  - "Nothing is downloaded, installed or started without the user's explicit confirmation; the assistant contacts only the local Ollama address and no other host"
  - "The same audit, status and suggestions are available through the control protocol (for example `ollama status`, `ollama suggest`), so an AI agent can run the setup check"
---

## Context
StudyFrog runs its local language model through Ollama (EPIC-055), but a user has to find out on their own which Ollama models their computer can run, install Ollama, start it, and pull a model. Many models are too large for a laptop's memory or video memory, and a model that does not fit makes the app slow or fails. The assistant should take that work over: look at the machine, look at Ollama, and say what to do next.

## Description
Add an assistant (a panel in the local-model settings and a screen reachable from the setup of the models) that:
- audits the hardware with read-only checks (CPU, memory, GPU and video memory where readable, free disk);
- queries the local Ollama service over its HTTP API (version, installed models, and the sizes the service reports) and reports its state;
- ranks a curated catalog of models by fit for this hardware, so the list changes when the machine or the installed set changes;
- walks the user through install, start, pull and run, step by step, with copyable commands for each operating system;
- sets the chosen model as the local model and proves it works with a short test prompt.

## Acceptance Criteria
- [x] The assistant audits this machine on request and shows CPU (cores and instruction-set support that matters for inference), total and free memory, the GPU and its video memory where it can be read (NVIDIA, AMD, Apple), and free disk space in the models folder; a value that cannot be read is shown as unknown, never guessed
- [x] The assistant queries the local Ollama service (by default at 127.0.0.1:11434) for its version and the models already installed, and shows whether Ollama is missing, not running, running with no models, or ready
- [x] The assistant lists models suggested for this machine, ranked by fit: each suggestion shows its download size, the memory it needs, whether it fits in memory or video memory, whether it is already installed, and a short reason for its rank; the list updates when the audit or the installed models change
- [x] The assistant guides the setup step by step from the state it found (install Ollama, start the service, pull a model, run it), with each command shown in a copyable block and the next step highlighted; the steps are the same whether the user is on Linux, macOS or Windows
- [ ] The user picks a suggested or installed model as StudyFrog's local model, and the assistant checks it with one short test prompt before it is used for scoring or grading
- [x] Nothing is downloaded, installed or started without the user's explicit confirmation; the assistant contacts only the local Ollama address and no other host
- [x] The same audit, status and suggestions are available through the control protocol (for example `ollama status`, `ollama suggest`), so an AI agent can run the setup check

## Design Notes
- Decided at build time (2026-11-23): the catalog is the curated `crates/studyfrog-service/data/ollama_catalog.toml`, shipped with the app (first open question). Pulling, installing and starting Ollama are shown as commands to copy and run by the user; the app runs none of them (a background pull was left out, so no progress bar or cancel exists yet). The app runs only read-only probes (`ollama --version`, `nvidia-smi`, `sysctl`, `vm_stat`, `df`, `/proc` and sysfs reads, each with a timeout) and HTTP GET to 127.0.0.1:11434 (the probe refuses any other host).
- The app does not start the Ollama service (second open question); the guide says `ollama serve`.
- A graphics card that does not report its memory (for example integrated graphics) is treated as having no video memory: its models fit only in free memory (third open question).
- The panel is on the Assistant screen, Connections tab, above the connection list (not in Settings). The control protocol answers `audit`, `ollama status` and `ollama suggest` from the same check.
- Build versus buy (P10): the parsers and the fit rule are written here; no hardware-probing crate was added. The HTTP GET reuses `ureq` through `studyfrog-net`, next to the POST that the agent already uses.
- The fit rule (`studyfrog-service::ollama::fit`): a model fits in video memory when its memory need is at most 90 percent of the free video memory, else it fits in memory when it is at most 80 percent of the free memory; a need or a memory figure that is unknown gives "unknown", which never fits. Fitting models rank first (video memory before memory), then those already installed, then larger models (more parameters).
- Windows: no value is read yet (every audit value shows unknown). macOS: Apple silicon shows a shared-memory GPU; Intel Macs show no graphics card.
- The chosen model is saved as the connection "Local model (Ollama)" only after the test prompt answers. The mock exam writes its questions and grades answers with the selected connection (`selected_provider`), so a model picked here reaches exam grading only after its test passed. A connection the user adds by hand is not tested by this assistant.
- The catalog sizes are approximate values from the Ollama library and are to be checked with `ollama show NAME` before a release.
- The hardware audit and the Ollama client and the ranking are pure where possible: parsing of command output and the fit rule are functions with unit tests on recorded samples; the probes (reading files, running `nvidia-smi` or `sysctl` with a timeout) are thin wrappers.
- The model catalog is a data file (TOML) with name, parameter count, quantization, download size and the memory each model needs. Ranking uses those numbers against the audited memory; a model is never ranked as fitting when its size is unknown.
- Ollama queries use the local HTTP API with short timeouts. A missing or stopped service is a state to show, not an error to hide.
- No network access beyond the local Ollama address. Pulling a model is a long operation that runs only after confirmation and shows progress and can be cancelled.
- No `unsafe`. Follow the layout rules of crates (no inherent impl blocks, files <= 250 lines); the audit and the Ollama client belong in a crate or module that the UI and the control protocol both use.

## Dependencies
- EPIC-055 (offline local SLM internal assistant)
- STORY-0083 (models for StudyFrog; the remote providers stay separate and are not used by this assistant)
- STORY-0124 (control protocol, for the agent-facing commands)

## Open Questions
- Catalog source: answered by the build. The curated catalog ships with the app (see Design Notes); a live query of the Ollama library is not done.
- Starting the service: answered by the build. The app never starts it; the guide shows `ollama serve`.
- GPUs without readable video memory: answered by the build. They count as no video memory, so the models fit only in free memory.

## Verification
Checked 2026-11-23 on Linux (4 logical cores, no GPU, no Ollama installed), in the running app (display :106, the app copy in `/tmp/claude-0/ollama`), with a stub of the Ollama API on 127.0.0.1:11434 (`/tmp/claude-0/ollama/stub.py`, not committed).
- Criterion 1 (audit): the Assistant > Connections panel "Check this computer" showed the processor (model, 4 logical cores, AVX2 and FMA yes), the memory (15.8 GB free of 16.9 GB), "Graphics card: none could be read" and the free disk of the models folder; `ctl audit` gave the same values. Screenshot `/tmp/claude-0/ollama/before-final.png`. Unit tests on recorded samples: `studyfrog-service/tests/ollama_audit.rs` (cpuinfo, meminfo, nvidia-smi with unreadable values, df, vm_stat, Mac flags). The NVIDIA, AMD and Apple GPU readers and the Windows probe are not checked on such hardware here: the GPU readers are checked on samples only, and Windows reads nothing (every value unknown).
- Criterion 2 (Ollama state): with no `ollama` program and no service the panel says "Ollama is not installed on this computer." (`before-final.png`, `ctl ollama status` gives `missing`). With the stub answering `/api/version` and `/api/tags`, it says "Ollama 0.5.4 is ready with 1 model(s)" (`/tmp/claude-0/ollama/after-final.png`). `tests/ollama_stub.rs` covers ready, no models, not running and missing. "No models" and "not running" were not seen in the window.
- Criterion 3 (suggestions): ranking and fit tests in `tests/ollama_fit.rs` (fitting models first, then installed, then larger; unknown size never fits). In the window the list shows the rank, the download and memory sizes, the fit pill, the installed mark and the reason. With the stub, the installed model moved to rank 1 among the fitting models (`ctl ollama suggest`), so the list changes with the installed models.
- Criterion 4 (guide): in the window, "1. Install Ollama" was highlighted as next, with its command in a block and a Copy button (`before-final.png`); with the service and a model present, the steps show "done" and "next" moves on. The step titles are the same for every system (`ollama_fit.rs`, `the_same_steps_on_every_system`); the install commands for macOS and Windows are not run here.
- Criterion 5 (test prompt before use): NOT CHECKABLE. This container has no Ollama and no model, so a real reply and a real pull cannot be seen. Against the stub, "Test and use qwen2.5:0.5b" showed "Test passed for qwen2.5:0.5b: ..." and saved the connection "Local model (Ollama)" in `providers.toml` with that model. `tests/ollama_panel_ui.rs` checks the order: the test first, the save only after a pass, nothing saved after a failure.
- Criterion 6 (nothing without confirmation): by code review and tests. The app runs only read-only commands (`ollama --version`, `nvidia-smi`, `sysctl`, `vm_stat`, `df`, reads of `/proc` and sysfs), each with a timeout; the install, pull and start steps are commands for the user to copy; the HTTP calls go to the loopback address only (`a_remote_address_is_never_contacted`). The test prompt runs only when the user presses its button. Not traced at the network level.
- Criterion 7 (control protocol): in the window `ctl audit`, `ctl ollama status` and `ctl ollama suggest` answered the same data as the panel (missing, then ready with the stub). Headless tests in `studyfrog-ui/tests/ollama_control.rs` (parsing and answer shapes); the help table lists the three rows.
- Gate: `scripts/check.sh` (see the commit message for its result).
