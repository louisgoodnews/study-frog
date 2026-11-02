# Familiarity and FSRS: a model for combining them

Research and proposal for TASK-0183 (STORY-0059). Written 2026-11-02 at the owner's request.
Simulation: [`familiarity-fsrs-sim.py`](familiarity-fsrs-sim.py) (standard library only).
Verification marks as in [learning-science](learning-science.md): **[v]** confirmed against the source's own
page or listing, **[u]** from memory or a secondary summary.

## 1. The requirement

The owner (2026-11-02): familiarity must influence FSRS so well-known cards are scheduled more generously and
poorly known ones less so. If a card is rated Easy but its familiarity is 0.5, FSRS should not just accept Easy and
schedule in 5 months but in about 2.5. (A figure of 30 days for familiarity 1.0 that came up earlier was an example,
never a cap.)

Familiarity (STORY-0058) is a value in 0.0 to 1.0 per item, driven by views, ratings and view time, with a slow fall
without contact. It is a *second opinion* about how well the user knows an item, built from signals FSRS does not see
(view time, views outside rehearsal) and from the same ratings FSRS sees.

## 2. What FSRS gives us to work with

FSRS-6 keeps per item a stability S (days for recall probability to fall to 90 %) and a difficulty D. Retrievability
is `R(t, S) = (1 + factor * t / S)^(-w20)` and the interval for a target retention r is
`I(r, S) = S / factor * (r^(-1/w20) - 1)` (`crates/studyfrog-srs`, `docs/specs/srs-algorithm.md`). After each review
the new stability is computed from the **actual elapsed time**, through the retrievability R at that moment:
`S' = S * (1 + gain)`, where the gain shrinks when the review happens early (R high) and grows when late.

Two consequences decide the design:

1. **Scaling the interval is coherent.** If familiarity shortens a planned interval, the card is reviewed at a higher
   R; FSRS then gives a smaller stability gain, which is the spacing effect it models. The algorithm needs no change to
   stay consistent with what happened.
2. **`R(t, S * m) = R(t / m, S)`.** A multiplier on stability is the same as a rescaling of time, so "scale the
   interval by m" and "treat the memory as m times as stable" are one operation.

## 3. Background

- **FSRS** itself uses only rating, elapsed time and its own state. The search done for this task found no published
  FSRS extension that feeds response time or an outside familiarity signal into the update; treat that as "not found",
  not "does not exist" **[u]**.
- **Half-life regression** (Settles and Meeder, 2016, "A Trainable Spaced Repetition Model for Language Learning",
  ACL, <https://aclanthology.org/P16-1174/>) **[v]** models the half-life of a memory as a weighted combination of
  features, `h = 2^(theta . x)`, and recall as `2^(-t/h)`. It is the precedent for a **log-linear** effect of outside
  features on memory strength; our multiplier `exp(beta * x)` is the same form.
- **Response time** is related to retrieval strength; Matuschak's notes record the observation that lower retrieval
  rates go with longer response times (<https://notes.andymatuschak.org/>) **[u]**. `learning-science.md` already lists
  latency as a hypothesis to validate.

## 4. Candidate models

| Id | Model | Verdict |
|---|---|---|
| M0 | Plain FSRS | The baseline. |
| M1 | **Absolute factor** on the interval: `m = exp(beta * (F - F0))` for a fixed neutral familiarity F0 | Rejected. F's scale is set by the familiarity formula, and a fixed F0 cannot be right for every stage of a card: a card with F = 0.5 is well ahead at its third review and far behind at its tenth. In the simulation it lost 0.1 to 0.7 points of recall at equal workload in most settings and never gained more than 0.25. |
| M2 | **Relative factor** on the interval: `m = exp(beta * (F - F_expected(S)))`, where `F_expected(S)` is the familiarity that a card with FSRS stability S usually has | **Recommended.** Reacts to *disagreement* between FSRS and familiarity, which is exactly the owner's example. FSRS state, its optimiser and its golden tests stay untouched. |
| M3 | Relative factor applied to the stability **gain** stored in the state | Not recommended. Changes the state FSRS trains on, so the optimiser and the golden tests stop applying; measured benefit at most +0.4 points. |
| M4 | **Rating override**: an Easy with low familiarity is treated as Good | Rejected. Discrete, and cannot express "5 months becomes 2.5". |

## 5. The proposed model (M2)

```
F_expected(S) = S / (S + S_half)                 // S_half = 30 days by default
m             = clamp(exp(beta * (F - F_expected(S))), m_min, m_max)
interval      = round(I(r, S) * m)               // never below 1 day, never above maximum_interval_days
                                                  // and never past an exam or due date
```

- `S` is the stability FSRS **stores**; it is never replaced by `S * m`. The next review updates it from the real
  elapsed time as it does today.
- Defaults to be confirmed by the evaluation in section 7: `beta = 2`, `S_half = 30`, `m_min = 0.25`, `m_max = 2`.
  `beta = 0` switches the model off and returns exactly the plain FSRS interval.
- `F_expected` should be learned per user: the mean familiarity of the user's own items grouped by stability.
  `S / (S + 30)` is the cold-start curve.
- **Evidence guard:** no adjustment before the third review of an item (`beta` ramps in from review 2 to 4). A new card's
  familiarity rises faster than `F_expected` for small S, which would otherwise double its first intervals (the table
  below shows 1.6 to 2.0 at S = 2.3). The simulation did not include this guard; it is part of the evaluation task.
- SM-2 uses the same factor with the current interval as S.

### Worked values (`beta = 2`, `S_half = 30`)

Factor `m` by FSRS stability (row) and familiarity (column). The 5 months to 2.5 months case of the owner is the
S = 150, F = 0.5 cell.

| S (days) | F_expected | F = 0.3 | 0.5 | 0.7 | 0.9 | 0.97 |
|---|---|---|---|---|---|---|
| 2.3 | 0.07 | 1.58 | 2.00 | 2.00 | 2.00 | 2.00 |
| 11 | 0.27 | 1.07 | 1.59 | 2.00 | 2.00 | 2.00 |
| 46 | 0.61 | 0.54 | 0.81 | 1.21 | 1.80 | 2.00 |
| **150** | 0.83 | 0.34 | **0.51** | 0.77 | 1.14 | 1.31 |
| 400 | 0.93 | 0.28 | 0.42 | 0.63 | 0.94 | 1.08 |

A well known card (F = 0.97) whose Easy rating FSRS had already turned into a long interval is stretched a further
31 % at S = 150; a card rated Easy that the user does not really know (F = 0.5) is halved. Every 0.1 of disagreement
is a factor of `exp(0.2)`, about 22 %.

## 6. Simulation

A synthetic learner with a known true memory (FSRS-6 dynamics with per-card ease, ratings with noise and a bias,
view time that falls with the true recall probability) reviews 400 cards for a year, three seeds, at target
retention 0.90. Workload is reviews per card; retention is the mean true recall probability over the year. "Gain"
compares each model with plain FSRS **at the same workload**, read from plain FSRS run at eight target retentions.

| Scenario | Plain: reviews, recall | M1 absolute | **M2 relative** | M3 gain | Control (no ratings, no view time) |
|---|---|---|---|---|---|
| A: user over-rates Easy | 5.9, 0.919 | -0.64 | **+1.33** | +0.16 | +1.56 |
| B: user rates accurately | 8.0, 0.941 | -0.11 | **+0.81** | +0.32 | +0.78 |
| C: view time says nothing | 5.9, 0.919 | -0.55 | **+1.35** | +0.43 | +1.56 |
| D: view time says a lot | 5.9, 0.919 | -0.65 | **+1.25** | +0.16 | +1.56 |

Values are gains in percentage points of mean recall at equal workload (M1 `beta` 2, F0 0.6; M2, M3 and control
`beta` 2, S_half 30). The full table with all parameters is the script's output.

**What the numbers say and do not say.**

- The *structure* matters: relative beats absolute in every scenario, and storing the adjustment in the state gains
  almost nothing. That is why M2 is recommended.
- The gain of M2 is **not evidence that ratings or view time add information.** The control, which gives the factor
  only the number of reviews and the time since them, does as well or slightly better (+1.56 against +1.33 in scenario A).
  In this simulation the model works as a brake on FSRS's fast interval growth, because the simulated memory grows
  slower than FSRS's default weights expect. Fitting FSRS's own weights to the data (the optimiser planned after M7)
  would remove much of that gain. Scenario D, in which view time is a strong signal, does not separate M2 from the
  control either, so the way familiarity uses view time (a mild change of the rating's step) is probably too weak to
  show, or the simulated gap is too small. This simulation cannot tell which.
- The learner is invented. It shows the mechanism works and which structure to choose; it cannot say how large the
  benefit is for a real person.

## 7. How to decide on real data

The app stores every review with its latency and rating, and will store view times (TASK-0134). That allows a test
on the user's own history instead of a simulation:

1. **Replay.** Walk the history in order. At each review compute FSRS's predicted recall `R` and the adjusted
   prediction `R_adj = R(t, S * m)` from the familiarity before the review. Score both against whether the user
   recalled (log loss and a calibration table in bins, as FSRS's own benchmark does).
2. **Verdict.** For a user whose history is long enough (at least 1000 reviews), fit `beta`, `S_half` and `F_expected` on
   the first 80 % and test on the last 20 %. The verdict says whether `R_adj` has lower log loss than `R`.
3. **Workload check.** Report the change in reviews per year the factor causes for the same history, so the user sees
   the trade-off. Use the simulator of the forecast report (`docs/specs/reports.md`) for it.
4. **Switch.** The model is **on from the start** (owner's decision, 2026-11-02) with `beta = 2`. Settings shows the replay
   verdict; if it says plain FSRS predicts better, Settings suggests turning the model off or using the fitted values. It
   never switches itself off.

## 8. Risks

- **Double counting.** Ratings feed both FSRS and familiarity. The relative form limits this because it reacts to the
  difference, but the replay in section 7 is what proves there is no net harm.
- **Feedback.** Shorter intervals mean more reviews, more views and a higher F. Views raise F only marginally, and the
  evidence guard and the clamp bound the effect.
- **Sparse data.** `F_expected(S)` needs many items; the cold-start curve covers the start and per-user fitting
  takes over once there are enough items.
- **Exam bound.** The factor never plans past an exam or due date; near an exam a factor above 1 is cut off.
- **Meaning drift.** If the familiarity formula (STORY-0058) changes, `beta` and `F_expected` must be fitted again.
  They belong to the same version of the specification.

## 9. Implementation sketch

In `studyfrog-srs`, as free functions (no inherent impls, P1), leaving `interval_days` and the golden tests alone:

```
pub struct FamParams { pub strength: f32 /* beta */, pub s_half: f32, pub min_factor: f32, pub max_factor: f32 }
pub fn familiarity_factor(fam: f32, stability: f32, reviews: u32, p: &FamParams) -> f32
pub fn interval_days_with(state: &MemState, factor: f32, p: &Params) -> u32   // exam bound applied by the caller
```

Properties to test: `strength = 0` gives factor 1; the factor never decreases when familiarity rises; it stays inside
the clamp; it is 1 when `F == F_expected(S)`; the interval is at least 1 day and at most the maximum.

## 10. Decisions of the owner (2026-11-02)

1. The relative reading (M2) is right: the factor depends on how familiarity compares with what FSRS expects, not on
   familiarity alone.
2. Defaults `beta = 2`, clamp 0.25 to 2 and no change before the third review are accepted.
3. The model is on from the start, not gated by the replay. This was against the recommendation to keep it off until the
   replay approves it, because the simulation cannot show that ratings or view time add information (section 6). The
   replay (section 7) is therefore advisory, and the per-stack switch and the strength setting are the safety valves.