---
id: EPIC-069
type: epic
title: "First-run setup wizard: vault, local model (Ollama), remote provider and the MiniLM model"
status: backlog
parents: []
labels: [m14, slm, ui, tooling]
acceptance_criteria:
  - "At first boot, after the vault is chosen, the user can set up the models in optional steps: local model via Ollama, remote model provider, the MiniLM embedding model, and a summary; every step can be skipped"
  - "The wizard suggests local models that fit this machine, using the audit and ranking of STORY-0206"
  - "The MiniLM model and any suggested model are downloaded from their source only after the user confirms each download, which shows its size and source"
  - "The wizard can be run again from Settings"
---

## Context
A new user meets the vault chooser first and has to find the local-model setup on their own. The models are optional
but make the assessment and exam features work (semantic answer comparison needs MiniLM; scoring and generation need a
local or remote model). Setting them up in one guided pass at first boot removes that gap.

## Description
Extend the first-boot flow with a sequence of optional steps. The wizard reuses the pieces that already exist: the vault
chooser, the hardware audit and Ollama status of STORY-0206, the remote providers of STORY-0083 and STORY-0084, and the
`ureq` download code in `studyfrog-net`.

## Acceptance Criteria
- [ ] Steps after the vault: local model (Ollama), remote provider, MiniLM, summary; each step can be skipped and the wizard can be left at any time
- [ ] Local-model step shows the STORY-0206 audit and the suggestions for this machine, with the reason for each rank
- [ ] Remote-provider step is opt-in and states that the text sent for scoring leaves the machine
- [ ] MiniLM and suggested model downloads each show size and source and start only on a click; progress and failure are shown; checksums are verified
- [ ] Wizard can be run again from Settings

## Design Notes
- Downloads use the Hugging Face source for the MiniLM model: `sentence-transformers/all-MiniLM-L6-v2` (config.json, tokenizer.json, model.safetensors) into `~/.local/share/studyfrog/models/all-MiniLM-L6-v2`, the folder the candle tests read. The same files were installed by hand on 2026-11-25.
- Step state is a pure function of settings and what is installed, so the flow can be tested without a window.

## Dependencies
- STORY-0206 (Ollama assistant, done), STORY-0083 and STORY-0084 (remote providers, done), TASK-0050 (candle embedder, done).

## Open Questions
- STORY-0206 says the assistant contacts only the local Ollama address. The wizard's downloads go to huggingface.co, which
  breaks that rule for the wizard. The owner accepted per-download consent; the STORY-0206 wording should be updated to
  say so before this is built.
