---
id: STORY-0206
type: story
title: "Ollama setup assistant: hardware audit, live Ollama status and model suggestions that fit this machine"
status: backlog
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
- [ ] The assistant audits this machine on request and shows CPU (cores and instruction-set support that matters for inference), total and free memory, the GPU and its video memory where it can be read (NVIDIA, AMD, Apple), and free disk space in the models folder; a value that cannot be read is shown as unknown, never guessed
- [ ] The assistant queries the local Ollama service (by default at 127.0.0.1:11434) for its version and the models already installed, and shows whether Ollama is missing, not running, running with no models, or ready
- [ ] The assistant lists models suggested for this machine, ranked by fit: each suggestion shows its download size, the memory it needs, whether it fits in memory or video memory, whether it is already installed, and a short reason for its rank; the list updates when the audit or the installed models change
- [ ] The assistant guides the setup step by step from the state it found (install Ollama, start the service, pull a model, run it), with each command shown in a copyable block and the next step highlighted; the steps are the same whether the user is on Linux, macOS or Windows
- [ ] The user picks a suggested or installed model as StudyFrog's local model, and the assistant checks it with one short test prompt before it is used for scoring or grading
- [ ] Nothing is downloaded, installed or started without the user's explicit confirmation; the assistant contacts only the local Ollama address and no other host
- [ ] The same audit, status and suggestions are available through the control protocol (for example `ollama status`, `ollama suggest`), so an AI agent can run the setup check

## Design Notes
- The hardware audit and the ranking are pure where possible: parsing of command output and the fit rule are functions with unit tests on recorded samples; the probes (reading files, running `nvidia-smi` or `sysctl` with a timeout) are thin wrappers.
- The model catalog is a data file (TOML) with name, parameter count, quantization, download size and the memory each model needs. Ranking uses those numbers against the audited memory; a model is never ranked as fitting when its size is unknown.
- Ollama queries use the local HTTP API with short timeouts. A missing or stopped service is a state to show, not an error to hide.
- No network access beyond the local Ollama address. Pulling a model is a long operation that runs only after confirmation and shows progress and can be cancelled.
- No `unsafe`. Follow the layout rules of crates (no inherent impl blocks, files <= 250 lines); the audit and the Ollama client belong in a crate or module that the UI and the control protocol both use.

## Dependencies
- EPIC-055 (offline local SLM internal assistant)
- STORY-0083 (models for StudyFrog; the remote providers stay separate and are not used by this assistant)
- STORY-0124 (control protocol, for the agent-facing commands)

## Open Questions
- Which catalog source: a curated list shipped with the app (reproducible, offline), or a live query of the Ollama library (current, needs network)? The default proposal is the curated list, refreshed in releases.
- Should the assistant start the Ollama service itself, or only tell the user how?
- How far should the ranking go for GPUs without readable video memory (for example integrated graphics): treat them as CPU-only, or ask the user?
