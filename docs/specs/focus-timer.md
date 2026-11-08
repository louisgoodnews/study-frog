# Focus timer (STORY-0112)

A Pomodoro-style timer in the top bar that runs on every screen and records each finished work block.
Pure state machine: `crates/studyfrog-ui/src/focus_timer.rs`; window: `focus_ui.rs`, `view/focus_bar.rs`;
record: `studyfrog_service::focus`, store table `focus_block` (schema v26).

## Phases and clock
- Work, short break, long break (after every fourth work block). Presets 25/5 (long 15), 50/10 (long 20),
  90/15 (long 30); custom work 1 to 600 minutes and break 1 to 120 minutes (long break three times the
  break, at most 240).
- Time comes from the shell's monotonic clock (`State::now`), advanced on the clock tick about once a second;
  pauses do not count. The wall clock (`State::wall_ms`) only stamps the start and end of a block.
- When a work block reaches its length it ends: it is recorded, the timer stops in the break phase and a
  card "Focus session complete (N min)" offers **Start break (N min)**, **Log in Diary…** and dismiss. A
  finished break stops in a fresh work phase ("The break is over."). Nothing starts by itself.

## Top bar
- Idle: a quiet stopwatch icon ("Focus timer"). Running or part-way: "Focus 24:12", "Paused 12:00" or
  "Break 04:10"; the hover text names the stack.
- Its menu: Start focusing / Pause / Resume, Skip the break, Reset, the presets, custom lengths, and the
  stack the focus counts for (any stack of the vault, or none).

## Record
- A finished block is stored in `focus_block(started_ms, ended_ms, minutes, stack, session_uid,
  session_date)`. It counts towards the calendar **study** session occurrence of that day that it overlaps
  most (local time at the block's end), if any; other event kinds are ignored. The notice says the
  minutes, the session and the minutes focused today.
- **Log in Diary…** opens the diary editor with an entry for today: the block's minutes as study time,
  its stack and a first line "Focus session of N minutes."; nothing is saved until the user saves it.
- Without an open vault the timer still runs but blocks are not recorded.
