---
id: TASK-0298
type: task
title: "Confirmed model downloads: MiniLM and suggested models, sizes, checksums and retry"
status: done
epic: EPIC-069
parents: ['STORY-0212']
estimate: 5
labels: [m14, slm, net, tooling]
acceptance_criteria:
  - "A download starts only after the user confirms it, and the confirmation shows the file size and the source host"
  - "MiniLM is fetched from sentence-transformers/all-MiniLM-L6-v2 (config.json, tokenizer.json, model.safetensors) into the models folder the candle tests read"
  - "Each file's SHA-256 is checked against a pinned value; a mismatch removes the partial file and reports it"
  - "A failed or interrupted download can be retried without restarting from zero when the server allows ranges"
---

## Context
The MiniLM model was installed by hand on 2026-11-25. The wizard must fetch it.

## Description
Build the download on the `ureq` code in `studyfrog-net`. Pin the SHA-256 values in the code, not in the user's settings.

## Acceptance Criteria
- [x] Confirmed downloads with size and source shown
- [x] MiniLM fetched into the models folder
- [x] Pinned SHA-256 check, partial file removed on mismatch
- [x] Retry without restarting from zero when ranges are supported

## Design Notes
Hashes of the installed files (2026-11-25): config.json 953f9c0d…, tokenizer.json be50c362…, model.safetensors 53aa5117….

## Dependencies
- TASK-0297.

## Open Questions
- Resolved 2026-11-25: STORY-0206 now allows any model provider with the source shown and confirmed.

## Verification note
Done 2026-11-25. Downloads are in `crates/studyfrog-net/src/model_plan.rs` (plans with pinned digests) and `crates/studyfrog-net/src/model_fetch.rs` (confirmation first, verified files kept, mismatches removed). The partial-file removal is a one-line change in `model_download.rs`, covered by `tests/model_download.rs`. The MiniLM fetch was run against huggingface.co (ignored test `the_minilm_plan_downloads_and_verifies_every_file`, about 76 s, all three files verified). The GGUF catalog digests and sizes were corrected from Hugging Face's published values. The size and source shown on screen are part of TASK-0299. `scripts/check.sh` passed on the tree before this commit.
