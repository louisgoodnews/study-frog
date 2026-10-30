# ADR 0004: SLM runtime

- Status: **Accepted** (2026-10-30); the M6 spike confirmed the latency budget (see Spike result)
- Date: 2026-10-30

## Context
Compare user text to reference text and return a score, locally, quickly (interactive).
The runtime must be swappable behind a trait.

## Options
| | candle | llama.cpp bindings | ort (ONNX Runtime) |
|---|---|---|---|
| Language/build | pure Rust | C++ build, FFI | prebuilt native lib, FFI |
| Embedding models (MiniLM-class) | yes | possible, less natural | yes, mature |
| Generative judge (small LLM, GGUF) | yes (quantised) | best support | via exported ONNX, awkward |
| Build/deploy simplicity | high | medium | medium (library distribution) |
| `unsafe_code = "forbid"` friendliness | in our code, yes | crate wraps FFI | crate wraps FFI |

## Decision (proposed)
**candle** first, embedding-based similarity as the primary score (fast, deterministic,
cheap), with an optional generative-judge path later. The trait:

```rust
trait SlmRuntime {
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SlmError>;
}
```
Scoring (cosine plus rubric adjustments, [scoring](../specs/scoring.md)) is pure free functions
over embeddings, so a runtime swap does not change scoring logic.

## Spike (M6, TASK-0050)
Measure p50/p95 latency for a 30-word comparison, memory, and model download size on a
reference machine; compare against `ort` with the same model. If candle misses the budget
(p95 > 150 ms CPU) switch to `ort`; only `studyfrog-slm` changes.

## Spike result (M6, 2026-10-30)
Reference machine: 16 logical CPUs, Linux, release build, candle 0.11 on the CPU, `all-MiniLM-L6-v2`
(F32 safetensors, 90.9 MB on disk), 256-token cap.

| Measure | Result |
|---|---|
| Model load | 60 to 90 ms |
| Memory | about +85 MB resident (19-23 MB before, 106-108 MB after) |
| Comparison of a 30-word answer with a 3-point rubric (5 texts, one batch) | p50 43 to 61 ms, p95 71 to 79 ms, max 81 to 116 ms (200 runs, varies with machine load) |
| Budget (p95 150 ms) | met with about half the budget to spare |
| Quality on 10 labelled pairs | semantic 10/10 at `min_score` 0.55; built-in lexical runtime 6/10 |

candle meets the budget, so there is no reason to switch to `ort` and the `ort` comparison was not run.
Debug builds are far slower; judge latency on release builds only.

The labelled set is tiny (5 paraphrases that should pass, 5 wrong answers). Correct paraphrases scored
0.65 to 0.97 and wrong answers 0.01 to 0.43, so the schema default `min_score` of 0.7 is too strict for the
semantic runtime (it would call some good paraphrases "partly right"); about 0.55 to 0.6 fits this sample.
Calibrating properly needs a bigger set. The default is unchanged until then.

The runtime lives in its own crate, `studyfrog-candle`, which only the application binary depends on; the
application passes its loader to the UI. This keeps the tensor library out of every other crate and test
binary (linking it into each UI test made the gate take about 40 minutes).

## Consequences
Models are downloaded/placed by the user into the app data dir; nothing is bundled.
