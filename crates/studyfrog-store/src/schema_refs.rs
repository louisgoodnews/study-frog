//! Schema step 20: items referenced by several stacks (ADR 0011).

/// `item_ref` holds what the files say (`referenced_by`, even when it is wrong, so the vault check can report it);
/// the view `membership` is the truth the services read: an item belongs to its primary stack (the stack whose
/// folder holds it) and to every existing stack that references it. Every file is read again once.
pub(crate) const V20: &str = "
CREATE TABLE item_ref (
    item_uid  TEXT NOT NULL REFERENCES item(uid) ON DELETE CASCADE,
    stack_uid TEXT NOT NULL,
    times     INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (item_uid, stack_uid)
) WITHOUT ROWID;
CREATE INDEX item_ref_stack ON item_ref(stack_uid);

CREATE VIEW membership AS
    SELECT s.uid AS stack_uid, i.uid AS item_uid, 1 AS is_primary
    FROM item i JOIN stack_identity s ON s.folder = i.stack_path
    WHERE i.kind <> 'stack'
    UNION ALL
    SELECT r.stack_uid, r.item_uid, 0
    FROM item_ref r
    JOIN stack_identity s ON s.uid = r.stack_uid
    JOIN item i ON i.uid = r.item_uid
    WHERE i.stack_path IS NOT s.folder;

UPDATE file SET mtime_ns = -1, hash = 0;
";
