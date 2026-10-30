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

## Not configuration
Per-item properties stay in front-matter ([markdown-schema](markdown-schema.md)): `id`, `kind`, `title`, `tags`,
`aliases`, timestamps, `reversible`, `rehearse_title`, `accepted`, `rubric`, `min_score`.
