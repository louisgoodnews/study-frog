# Config files

Settings live in TOML, content in Markdown ([ADR 0007](../adr/0007-settings-in-toml-files.md)). Unknown keys and
tables are **errors** (a typo must not silently do nothing). Comments are allowed. Parsing and validation are in
`studyfrog-config`; reading the files is `studyfrog-store`'s job.

## `studyfrog.toml` (vault root, optional)
```toml
[srs]
scheduler = "fsrs"            # "fsrs" (default) or "sm2"; the default for stacks that do not choose
request_retention = 0.9       # FSRS target retention, strictly between 0 and 1
maximum_interval_days = 36500 # at least 1
weights = [0.212, ...]        # optional: exactly 21 finite numbers (FSRS-6); default = published weights
```
Missing file or missing keys mean the defaults shown. An invalid file is reported with its line and the app
falls back to the defaults.

## `_stack.toml` (next to a stack's `_stack.md`, optional)
```toml
scheduler = "sm2"             # overrides the vault default
request_retention = 0.85      # FSRS retention override
daily_new_limit = 20          # new items per day
daily_review_limit = 200      # reviews per day
```
Every key is optional; omitted keys use the vault defaults. A `_stack.toml` without a `_stack.md` in the same
folder is ignored. Daily limits count distinct items per day from the review history (see
[storage-schema](storage-schema.md)); a stack switched from FSRS to SM-2 (or back) keeps its items scheduled.

## `achievements.toml` (vault root, optional)
```toml
[[achievement]]
id = "week"                   # one word, unique; remembers the unlock
title = "A week in a row"
description = "Review something on 7 days in a row."   # optional
scope = "global"              # "global" or "stack" (unlocks separately for every stack)
metric = "streak_days"        # runs_finished (global only), items_rated, easy_ratings, items_learned,
                              # items_mastered, streak_days
threshold = 7                 # at least 1
```
When the file exists it **replaces** the built-in set (an empty file means no achievements); an invalid file
is reported and the built-in set is used. Mastered means scheduled 21 days or more ahead. Only runs that
update the schedule write reviews, so practice runs do not count towards review-based metrics.

## `app.toml` (`$XDG_CONFIG_HOME/studyfrog/`, written by the app)
```toml
vault = "/home/me/notes"       # the vault opened last
recent = ["/home/me/notes"]    # recently opened vaults, newest first
reading_wpm = 215              # measured reading speed, 50 to 1000; absent = the default of 238
```
Further keys, all optional:
```toml
answer_engine = "words"        # "auto" (default: language model when installed) or "words" (word matching only)
reading_fast_percent = 75      # 10 to 100: reading under this share of the expected time is fast
reading_slow_percent = 200     # 100 to 1000: reading over this share is slow
reading_min_words = 8          # 1 to 200: shorter answers say too little about familiarity
```
`reading_wpm` is set by the reading-speed test on the Settings screen ("Use the default speed" removes it).
A value outside 50 to 1000 is an error with its line.

## Not configuration
Per-item properties stay in front-matter ([markdown-schema](markdown-schema.md)): `id`, `kind`, `title`, `tags`,
`aliases`, timestamps, `reversible`, `rehearse_title`, `accepted`, `rubric`, `min_score`.

## `subjects.toml` (vault root, optional)
Teachers and subjects, edited on the Settings screen:
```toml
[[teacher]]
name = "Ms Rivera"

[[subject]]
name = "Mathematics"
teacher = "Ms Rivera"   # optional; a subject has at most one teacher
```
Names are unique (ignoring case), one line, without commas. Unknown keys and unknown teachers are errors, and the
file is then not used (the Settings screen says why). A stack lists its subjects in its front matter:
`subjects: Mathematics, Physics`.

## `profiles.toml` (config folder)
```toml
current = "School"

[[profile]]
name = "School"
```
"Default" always exists and keeps `app.toml` and `secrets.toml` directly in the config folder; every other profile
keeps its own in `profiles/<name as a folder>/`. Names are unique ignoring case, one line, at most 40 characters.

## Backups
A backup is a plain tar file: `manifest.toml` first (layout version, time, program version, vault name, file count
and size, whether settings and keys are inside), then `database/index.sqlite` (a consistent copy made with
`VACUUM INTO`), `vault/<path>` for every vault file except the index folder, and `config/app.toml` and
`config/secrets.toml` when included. Restoring writes into a new or empty folder only; paths that leave it are refused.
