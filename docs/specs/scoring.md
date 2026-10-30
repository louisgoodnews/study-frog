# SLM comparison scoring contract

## Input
`reference: &str`, `answer: &str`, optional `rubric: &[&str]`, optional `accepted: &[&str]`.

## Output
```
Score { value: f32 /* 0..=1 */, verdict: Correct|Partial|Incorrect, rubric_hits: SmallVec<bool> }
```

## Algorithm (v1, embedding-based)
1. Normalise both texts (Unicode NFC, trim, collapse whitespace; lowercase only for comparison).
2. Embed answer, reference and each accepted alternative via `SlmRuntime::embed`.
3. `value = max over {reference, accepted} of cosine(answer, candidate)`, clamped to [0, 1].
4. If a rubric is present, each point is a hit when `cosine(answer, point) >= point_threshold`;
   final `value = 0.6 * value + 0.4 * hit_ratio`.
5. Verdict: `Correct` if `value >= min_score`; `Partial` if `value >= min_score - 0.2`; else `Incorrect`.

## Properties (proptest)
- Range: always in [0, 1]. Identity: `answer == reference` gives 1.0 (within epsilon).
- Empty answer gives 0. Score does not depend on surrounding whitespace.
- Pure: same inputs and same embeddings give the same output.

## Mapping to SRS
Verdict suggests a rating (Correct→Good, Partial→Hard, Incorrect→Again); the user can override.

## Non-goals
Factual verification beyond the reference text; thresholds are calibrated in M6 against a labelled set.
