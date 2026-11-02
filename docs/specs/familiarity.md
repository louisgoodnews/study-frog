# Familiarity

Planned for 0.3.0 (STORY-0058). The meaning was defined by the owner on 2026-11-02, who also accepted the behaviour on the worked examples
(section 8), the note cap, the mark factors and the one-view-per-hour rule that day. The constants below are the starting
values; the validation of TASK-0137 may calibrate them.

## 1. Meaning and range
Familiarity is **how familiar the user is with an item**: a card, a note, a question or a stack. It is a number from
**0.0 to 1.0**; 1.0 means the user knows it very well and need not rehearse it in the near future. The UI shows it as a
percentage (`0.91` is "91 %"). It is a sub-criterion of scheduling and a per-item report of how well the user knows an
item ([familiarity-and-fsrs](../research/familiarity-and-fsrs.md)); it is not a probability of recall.

It behaves as the owner described: it grows marginally with every view; grows significantly with an easier rating;
falls significantly with a harder one; grows the shorter the view time and falls the longer; and falls slowly without
contact unless the item is finished.

## 2. The value is a fold over the history
`familiarity(events, now, context)` is a **pure function**: it starts at 0.0 at the first event and folds the events in
time order, applying the decay of section 4 before each one and once more up to `now`. No stored state, no clock inside.

## 3. Events and what each does
`F` is the value before the event, `s` the speed factor, `m` the mark factor. Gains move F towards 1, losses scale it down.

| Event | Change |
|---|---|
| Rating **Easy** | `F += 0.45 * (1 - F) * s * m` |
| Rating **Good** | `F += 0.30 * (1 - F) * s * m` |
| Rating **Hard** | `F -= 0.25 * F` |
| Rating **Again** | `F -= 0.50 * F` |
| **View** of the item page (no rating) | `F += 0.02 * (1 - F) * s_v` |
| **Edit** | no change; it counts as contact, so the decay clock restarts |

- **Speed factor `s` (ratings Easy and Good):** `s = clamp(expected / actual, 0.5, 1.5)`. `actual` is the time the answer
  side was on screen (the measurement the reading-speed suggestions already use); `expected = max(3 s, 60 * words / wpm)` for
  the answer text at the user's reading speed (default 238 words a minute). Faster than expected raises the gain, slower
  lowers it; a rating without a measured time uses `s = 1`.
- **View factor `s_v`:** `clamp(expected / actual, 0.25, 1.5)` for the item's own text. Views under 1 second are ignored and
  at most one view per item per hour counts. A view during a run is the rating's, not a second event.
- **Mark factor `m`:** the item's difficulty mark scales gains: `easy` 1.1, `medium` 1.0, `difficult` 0.8; no mark is 1.0.
- **Priority** does not change an item's own value; it weights the item in a stack's roll-up (section 6).
- Losses ignore the speed: a lapse is a lapse however long it took.

## 4. Fall without contact
Between two events, and from the last event to `now`: `F *= 0.5 ^ (days / 365)`. A year without contact halves it.

**The fall pauses** (the owner, 2026-11-02) for the stretch from the last event to `now` while the item's status is
`done` ([status](status.md)), and in that stretch for the days after the exam date of the item's stack. Earlier gaps always
fall: the status history is not stored, so a gap in the past is treated as ordinary. When a status log exists, gaps can be
paused retroactively. Items skipped by an ancestor stack being `done` or `paused` follow the same rule as their own status.

## 5. Notes and the confidence flag
A note has views and edits but no ratings, so its value rests on views only and is **capped at 0.8**: reading a note
does not show that it is known. Cards and questions are not capped.

**Confidence** says how much history stands behind the value: cards and questions `min(1, ratings / 5)`; notes
`min(0.5, views / 10)`; a stack the weighted mean of its items. Below 0.4 the UI shows "few reviews" beside the value.

## 6. Stacks
`F(stack) = sum(w_i * F_i) / sum(w_i)` over every item in the stack and its sub-stacks, with the weight `w_i` by priority:
lowest 0.5, low 0.75, medium 1, high 1.5, highest 2. An empty stack has no value. Items on hold count; the roll-up is
derived, never stored.

## 7. Constants
All constants live in one record (`FamParams`) with these defaults, so calibration changes data, not code:
gains 0.45 and 0.30, losses 0.25 and 0.50, view gain 0.02, half-life 365 days, speed bounds 0.5 and 1.5, view bounds 0.25 and
1.5, note cap 0.8, mark factors 1.1 and 0.8, confidence 5 ratings and 10 views.

## 8. Worked examples
**A card over ten reviews** (day, rating, speed factor `s`; F after the event, with the fall between events applied):

| Day | Rating | s | F before | F after |
|---|---|---|---|---|
| 0 | Good | 1.0 | 0.00 | 0.30 |
| 2 | Good | 1.0 | 0.30 | 0.51 |
| 6 | Easy | 1.5 | 0.51 | 0.84 |
| 15 | Good | 1.0 | 0.83 | 0.88 |
| 40 | Hard | 1.0 | 0.84 | 0.63 |
| 43 | Good | 1.0 | 0.62 | 0.74 |
| 60 | Good | 1.0 | 0.71 | 0.80 |
| 110 | Easy | 1.5 | 0.73 | 0.91 |
| 170 | Good | 1.0 | 0.81 | 0.87 |
| 260 | Easy | 1.5 | 0.73 | **0.91** |

After 90 days without contact the value is 0.77, after a year 0.46. If the item is `done` at day 260 it stays 0.91. A lapse
(Again) on day 300 instead brings it from 0.87 to 0.42.

**The same rating, different speed.** A first rating of Easy gives 0.68 when the answer was read fast (`s` 1.5), 0.45 at the
expected time and 0.23 when it was slow (`s` 0.5); a first Good gives 0.45, 0.30 and 0.15. This is the owner's rule that
a shorter view time means more familiarity.

**A note:** ten views three days apart give 0.18, thirty give 0.42; it never passes 0.8.

## 9. Properties the implementation must have
Value always inside 0.0 and 1.0; a view or a gain rating never lowers the value; Easy raises it at least as much as Good,
Good more than a view; Hard and Again lower it, Again at least as much as Hard; a shorter time never gives a lower value
than a longer one for the same rating; without events the value does not rise; the value is the same for the same history
whatever the order the events were stored in (they are sorted by time); the fall pauses exactly as in section 4.

## 10. Interface and budget
Free functions in the report crate (no inherent impls): `familiarity(&[FamEvent], now, &FamContext, &FamParams) ->
Familiarity { value, confidence }` and `stack_familiarity(&[(Priority, Familiarity)]) -> Option<Familiarity>`. A
`FamEvent` is a rating (with rating, answer time, expected time), a view (with duration and expected time) or an edit,
each with a time. The cost is linear in the events; **10 000 items with 20 events each must be valued in under 20 ms**
(measured by a benchmark; the number is a budget to confirm).

## 11. Validation
On the labelled reviews of the simulated history that the forecast uses, check that items later recalled have a higher
value than items later forgotten (area under the ROC curve) and record the number and the date in
[reports](reports.md). The effect of familiarity on intervals is validated separately by the replay of TASK-0184.
