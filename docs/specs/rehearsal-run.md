# Rehearsal run

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
| `RunFinished` | normal end; `RunAborted` for early exit |

## Latency capture
- Measured with a monotonic clock injected by the shell (`Clock` trait); never wall-clock.
- Latency = `t(event) - t(ItemShown) - paused_time_within_item`.
- Stored as integer milliseconds.

## State machine
`Idle → Running ⇄ Paused → Finished|Aborted`. Transition is a pure function
`step(RunState, Input) -> (RunState, Vec<RunEvent>)`; invalid inputs return an error and leave
state unchanged.

## Persistence
Events are append-only rows in SQLite (`run_event`), written in one transaction per item.
A run's aggregates (counts, median latency) are derived, not stored. A crashed run is
recovered as `Aborted` on next start.
