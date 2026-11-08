# Rehearsal run

## Run plan
A run is built from a **plan**; rehearsal is not only spaced repetition. The plan is persisted with the run.

| Part | Values |
|---|---|
| `order` | `Due` (spaced: most overdue first, then new items), `Sequential` (library order), `HardestFirst`, `EasiestFirst` (by FSRS difficulty), `StalestFirst` (least recently reviewed, never-reviewed first), `LowestRetrievability` (predicted recall, lowest first) |
| `filter` | kinds (note, card, question), stacks, tags (any of), state (`Any`, `New`, `Learned`, `Due`, `NotDue`), difficulty range |
| `limit` | optional cap on the queue length |
| `requeue_again` | an item rated Again returns to the end of the queue (at most 3 times per item per run) |
| `effect` | `UpdateSchedule` (ratings feed SRS and write reviews) or `PracticeOnly` (events and latency are recorded, the schedule is untouched) |
| `shuffle` | put the chosen items in a random order (seeded, reproducible); independent of `order` and `effect` |
| `seed` | drives `shuffle` (and tie-breaking where randomness is used) |

Semantics:
- Only `Due` order uses the daily new/review budgets and hides items that are not due. Every other order
  goes through the **whole filtered list until it is exhausted**.
- Shuffling only randomises the sequence of the items the order chose. It works with every order, spaced
  included, and never changes the effect: a shuffled spaced run still updates the schedule. The limit cuts by
  the order's priority first and the cut items are then shuffled; a sequential run has no priority, so there
  the shuffle comes first and the limit picks a random sample.
- Defaults (confirmed by the owner): spaced runs `UpdateSchedule` with `requeue_again`; the presets that go
  through everything (in order, shuffled) and the drill presets are `PracticeOnly`. The user can override `effect` (for example a sequential first pass over new material
  that should start the schedule).
- Rationale for `PracticeOnly`: cramming outside the scheduled spacing would distort the memory model's
  inputs (design judgement, see learning-science notes); latency data is still valuable, so events are always recorded.
- A difficulty filter or order needs an FSRS memory state; items without one (new items, SM-2 items) are
  excluded by the filter and sorted last by the orders.
- Queue building is a pure function of `(candidates, plan, now, budgets)`; ordering is deterministic.

## Pacing: free, countdown and count up
A plan also has a **pacing** (STORY-0070). It says how the clock figures in the run; it never changes which items are chosen or what a rating does.

| Pacing | Stored as | Meaning |
|---|---|---|
| `Free` | `free` | The run as it always was: no limit, the elapsed time is not shown. The default, and what every run made before 0.3.0 reads as. |
| `Countdown(seconds)` | `countdown` and `countdown_secs` | A time the user sets. The run ends by itself when it is up. |
| `CountUp` | `count_up` | No limit. The elapsed time is shown, and the run goes on until the last item of the list is completed or the user ends it. |

- **The clock** is *active time*: the monotonic time since the start minus all time paused. Pausing stops it; a paused run does not run out.
- **Countdown limits** are 1 to 180 minutes (in seconds in the plan); the setup offers 5, 10, 15 and 25 minutes and a custom amount. A countdown cannot be
  extended once it has started (proposal; the owner may allow it).
- **When the time is up** the engine ends the run: a `TimeUp` event (carrying the item on screen, if there is one), then `RunFinished`. **The item on
  screen counts as not answered** (the owner, 2026-11-02): it gets no rating and changes no schedule, whether or not its answer had been shown. Items
  rated before keep their effect. Any input that arrives at or after the limit, a rating included, is *not applied*: the clock owns that second.
- **Ticks:** the shell sends a `Tick` input about once a second; it moves the clock and does nothing else, so a countdown ends on time even if the
  user does nothing. While paused a tick does nothing.
- **A queue that is used up before the time** finishes the run early, as always; requeued items (rated Again) keep coming back until the time is up.
- **Warnings** (screen, not engine): the time left is shown, and at 60 and at 10 seconds a message says so in text and is announced to screen readers; colour is never the
  only cue. The engine offers `remaining_ms(session, t)`.
- **Count up** needs nothing special from the engine: the screen shows `active_ms(session, t)`; the run ends in the usual ways (queue done, `Abort`).
- **Reports and the review** show the pacing, the planned time (countdown), the time used (`run_active_ms` over the events) and the answers rated per minute.
- **Storage:** `run.pacing` (`free`, `countdown`, `count_up`) and `run.countdown_secs` (migration 16). A crashed run is recovered as `Aborted` and keeps its pacing.

## Events
All events carry `run_id`, `item_id`, `t_ms` (monotonic ms since run start) and a sequence number.
| Event | Meaning |
|---|---|
| `RunStarted` | queue fixed, mode recorded |
| `ItemShown` | prompt displayed; starts the item timer |
| `ItemSkipped` | user skipped; latency = time-to-skip |
| `ItemFlipped` | answer revealed; latency = time-to-flip |
| `AnswerSubmitted` | free text submitted; latency = time-to-answer; text stored |
| `ItemRated` | Again/Hard/Good/Easy chosen (feeds SRS) |
| `RunPaused` / `RunResumed` | paused time is excluded from latency |
| `TimeUp` | a countdown ran out; carries the item on screen, if any; followed by `RunFinished` |
| `RunFinished` | normal end; `RunAborted` for early exit |

## Latency capture
- Measured with a monotonic clock injected by the shell (`Clock` trait); never wall-clock.
- Latency = `t(event) - t(ItemShown) - paused_time_within_item`.
- Stored as integer milliseconds.

## State machine
`Idle → Running ⇄ Paused → Finished|Aborted`. A `Session` (run, queue, cursor, timers) is advanced by
`step(&mut Session, Input) -> Result<StepOutput, StepError>`; `Input` carries the monotonic time.
Invalid inputs return an error and leave the session unchanged. Showing the next item and finishing the
run are automatic. Inputs: Start, Flip, Skip, SubmitAnswer, Rate, Pause, Resume, Abort.

## Persistence
Events are append-only rows in SQLite (`run_event`), written in one transaction per item.
A run's aggregates (counts, median latency) are derived, not stored. A crashed run is
recovered as `Aborted` on next start.

## The rehearsal screen (M4)
Distraction-free: the prompt, a progress counter and the time on the current item, nothing else.

| Key | Effect | When |
|---|---|---|
| Space | reveal the answer | prompt shown |
| 1 2 3 4 | rate Again, Hard, Good, Easy | answer revealed |
| S | skip the item | prompt shown |
| P | pause / resume (pausing hides the question and stops the item timer) | running / paused |
| Esc | stop the run (it is saved as aborted) | any time |

- Shortcuts are off while a text field has the keyboard (answer mode), so typing never triggers them.
- Answer mode: type the answer and submit; the reference answer is then shown for comparison (SLM scoring arrives in M6).
- The engine time is the egui time written into `State::now` each frame; the pure `update` calls the engine's `step`
  with it, and every step is saved through `studyfrog-service::record_step`.
- Item content is read from the file on demand, three items ahead of the current one; a file that cannot be read
  shows a message and the run continues.
- When a run is over the screen shows a summary (counts per rating, skips, median time to answer) and, for
  practice-only runs, says the schedule was not changed.
- Runs left open by a crash are recovered as aborted the next time the vault is opened.


## Marks, reading speed and the review
- **Marks.** While an item is on screen (before or after the reveal) the user may mark it Easy, Medium or
  Difficult (keys E, M, D). Each mark is an `item_marked` event; the last one counts. Marks are about how the
  item felt and do not change the rating or the schedule.
- **Reading time.** The time between revealing the answer and rating it, minus paused time. Baseline: 238
  words per minute (average silent reading of non-fiction); words are whitespace-separated. Backs of fewer
  than 8 words are ignored.
- **Reading speed.** The baseline is the user's measured speed (`reading_wpm` in `app.toml`) or 238. The
  Settings screen measures it: the user reads a lorem ipsum text of about 120 words, says when finished, and
  the speed is words divided by time; a result outside 50 to 1000 words per minute is rejected as not being
  reading. A run uses the speed in force when it started.
- **Suggested rating.** Only a clearly fast or slow read suggests anything. Fast (under 75 % of the expected
  time): a known item rated Good is suggested Easy ("rather easy for you"); a new item rated Good or Easy is
  suggested Hard (it went by too fast to have been read). Slow (over 200 % of the expected time): a struggle
  shows even if the answer came, so Good is suggested Hard and Easy is suggested Good. Ratings of Again or
  Hard are never softened.
- **The user's mark.** A mark also suggests a rating, and wins over reading speed when both apply: marked
  difficult but rated Good or Easy suggests Hard; marked easy and rated Good suggests Easy. Medium or no mark
  suggests nothing, and a low rating is never softened.
- **Thresholds.** Fast, slow and the minimum word count can be changed in `app.toml` (see
  [config-files](config-files.md)).
- **Answer match.** In answer mode the review shows how each typed answer compared with the reference; the
  comparison is stored per run (schema v6).
- **Review.** After the run, each item shows its previous rating, the user's mark and rating, and the
  suggestion with the reason and the reading time. In a schedule-updating run the user can apply a
  suggestion: the rating of the item's latest review is replaced and its schedule rebuilt by replaying the
  review history, which gives exactly the state the other rating would have produced then. The event log
  keeps what the user pressed. Practice-only runs show suggestions but cannot apply them.

## Priority and the owner's difficulty
Every item has a **priority** (`lowest`, `low`, `medium`, `high`, `highest`; default medium) and may have a
**difficulty** word (`easy`, `medium`, `difficult`). Both come from the item's own front matter, else the nearest
stack above that states one. In queue building:
- **Spaced runs** list due items and then new items by priority first (highest first), so a daily allowance goes
  to the more important items. A high priority makes an item due 12 hours early, the highest 24 hours early, and
  the lowest only 12 hours after its due time.
- **Other orders** keep their own key (hardest, easiest, stalest, lowest recall); priority breaks ties before
  the library position. Sequential keeps the library order.
- **Hardest and easiest first** use the owner's difficulty (2.5, 5.5, 8.5 on the 1 to 10 scale) for items that were
  never reviewed and so have no memory difficulty.

## Exam mode
`RunMode::Exam` is a strict, summative run: written answers only, no skipping, no going back, no peeking, no
rating, answers locked on submission, and a percentage with a grade at the end. Its rules, scoring and grade
scale are specified in [exam-mode.md](exam-mode.md).
