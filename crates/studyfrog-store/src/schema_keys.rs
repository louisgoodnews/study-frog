//! Schema step 24: item keys and colours (STORY-0096, STORY-0097).

/// `item.key` and `item.color` mirror the `key` and `color` of each file's front matter (derived, so
/// every file is read again once). `key_counter` remembers the highest number handed out per prefix,
/// so a key of a deleted item is never handed out again; it is authoritative, not derived.
pub(crate) const V24: &str = "
ALTER TABLE item ADD COLUMN key TEXT;
ALTER TABLE item ADD COLUMN color TEXT;
CREATE INDEX item_key ON item(key COLLATE NOCASE);
CREATE TABLE key_counter (
    prefix TEXT PRIMARY KEY,
    last   INTEGER NOT NULL CHECK (last >= 0)
) WITHOUT ROWID;
UPDATE file SET mtime_ns = -1, hash = 0;
";
