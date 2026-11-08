# Settings

Open **Settings** with the gear button in the top bar or from the sidebar. The screen has tabs: **Study and
scheduling**, **Rehearsal**, **Answers and AI**, **Data** and **Subjects and teachers**. The **Left** and **Right**
arrow keys move between tabs, and the last tab you used is remembered.

Some settings are changed on this screen. Others live in small text files you edit yourself; the screen shows their
current values and says which file to edit.

## Study and scheduling

- **Scheduling** shows the values in force: **Default scheduler** (FSRS or SM-2), **Target retention (FSRS)**,
  **Maximum interval (days)**, **Familiarity strength**, **Familiarity threshold** and **FSRS weights**. If
  `studyfrog.toml` has a mistake, the screen says so and the defaults are used.
- **Record how long I look at items**: untick to stop StudyFrog timing how long you look at an item page. See
  [[privacy]].
- **Does familiarity help?**: StudyFrog replays your own review history and tells you whether familiarity improves
  the predictions of what you will recall. With at least 1 000 reviews it also finds the best values for you and
  how many more or fewer reviews you would have. It never switches anything itself. See [[familiarity]].

## Rehearsal

**Reading speed**: press **Measure my reading speed**, read the text at your normal pace and press **I have finished
reading**. Then **Use this speed**, **Discard** or **Try again**. **Use the default speed** goes back to 238 words a
minute. Rehearsal uses your speed to suggest easier or harder ratings. See [[rehearse]].

## Answers and AI

**Answer comparison**: **Language model when installed** or **Word matching only**. The language model compares
meaning better; it is used only if installed (see the [[faq]]). Either way, a high match means similar wording or
meaning, not that the facts are right. The fast and slow reading thresholds are shown here too.

The AI assistant has its own screen: [[assistant]].

## Data

- **Backup and restore**: **Back up now…** writes one `.tar` file with your notes, pictures, PDFs, review history,
  board, calendar and settings. Tick **Include API keys** to add the assistant's keys (they are not encrypted in the
  backup). **Restore from a backup…** restores into a new or empty folder; nothing existing is overwritten. Tick
  **Also replace my settings with the ones in the backup** if you want those too; your old settings are kept next to
  them.
- **Import and export**: **Export…** and **Import…** open the import and export dialog. See [[import-export]].

## Subjects and teachers

Add teachers (**Add teacher**) and subjects (**Add subject**), and choose who teaches each subject (**Taught by**).
A subject has at most one teacher. A stack can have several subjects, chosen on its item page, and what is inside a
stack takes its subjects. Names may contain commas. A name that cannot be stored is refused before saving, with the
reason next to the field. They are saved in `subjects.toml` in the vault.

## Profiles

Profiles are in the person button at the right of the top bar. Each profile has its own settings, recent vaults and
API keys. **Switch to**, **Create profile** and **Delete** a profile there. Deleting removes the profile's settings
and keys, not its vault folder. **Close vault** is in the same menu.

## The settings files

`studyfrog.toml` in the vault folder holds the vault's defaults. Every key is optional:

```toml
[srs]
scheduler = "fsrs"            # or "sm2"
request_retention = 0.9       # FSRS target retention, between 0 and 1
maximum_interval_days = 36500
familiarity_strength = 2.0    # 0 to 10; 0 switches familiarity off
familiarity_threshold = 0.9   # due items at or above this familiarity are left out
```

`_stack.toml` next to a stack's `_stack.md` overrides them for one stack:

```toml
scheduler = "sm2"
request_retention = 0.85
daily_new_limit = 20          # new items per day
daily_review_limit = 200      # reviews per day
familiarity = false           # switch familiarity off for this stack
```

`app.toml` in your config folder (usually `~/.config/studyfrog/`) is written by the app. You may add:

```toml
answer_engine = "words"       # or "auto"
track_views = false
calendar_reviews = false      # hide planned reviews in the calendar
reading_fast_percent = 75     # 10 to 100
reading_slow_percent = 200    # 100 to 1000
reading_min_words = 8         # 1 to 200
```

A key StudyFrog does not know is an error, reported with its line, so a typo never silently does nothing. Files are
read when they are needed, so there is no need to restart.
Other files: `achievements.toml` ([[achievements]]) and `filters.toml` ([[library]]).

## Related pages

[[familiarity]], [[privacy]], [[faq]], [[glossary]].

## Appearance
The **Appearance** tab changes how StudyFrog looks and starts. Every change shows at once and is saved; you never need to
restart.

- **Theme**: Dark, Light, or Follow system (it changes with your desktop). The profile menu has the same choice.
- **Interface size** makes everything larger or smaller (70 to 200 %); **Ctrl and plus**, **Ctrl and minus** and **Ctrl
  and 0** do the same from any screen. **Text size** changes only the text. Longer
  text (notes, help pages, answers) is kept to about 70 characters a line, however wide the window, so it is easy to
  read; the column grows with the text size.
- **Density**: Compact, Comfortable or Spacious: the room between controls, inside cards and between groups.
  Cards are set off by a lighter (or, in the light theme, whiter) tone rather than outlines; **High contrast** draws
  outlines around controls and cards again.
- **Accent colour**, **High contrast** and **Reduce motion**. All six accents look equally bright, and every text
  colour is checked for contrast in both themes, with and without high contrast.
- **Open this screen first**: Home, Library or the screen you used last. **Start with the sidebar folded away.**
  **Zen mode during rehearsal and exam runs** hides the sidebar and the status bar while a run is on screen
  (Ctrl+Shift+F or F11 leaves it, and it comes back with the next run). **Remember the window's size and place.** Some desktops do not tell the place of a window; then only its size is kept.
- **Restore the default appearance** puts every choice back.
