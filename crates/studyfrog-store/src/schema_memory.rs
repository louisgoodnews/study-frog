//! Schema step 21: memory per stack (ADR 0011, TASK-0218).

/// Reviews, FSRS state and SM-2 state are kept for each (item, stack). Every row that exists becomes the row of the
/// item's primary stack (the stack whose folder holds it), or of the Inbox (uid 0) for a loose item or an item the
/// index does not know. Nothing is lost: each old row becomes exactly one new row.
pub(crate) const V21: &str = "
ALTER TABLE review ADD COLUMN stack_uid TEXT NOT NULL DEFAULT '00000000000000000000000000';
UPDATE review SET stack_uid = COALESCE(
    (SELECT s.uid FROM item i JOIN stack_identity s ON s.folder = i.stack_path WHERE i.uid = review.item_uid),
    '00000000000000000000000000');
DROP INDEX review_item;
CREATE INDEX review_item ON review(item_uid, stack_uid, at_ms);
CREATE INDEX review_stack ON review(stack_uid, at_ms);

CREATE TABLE mem_state_v21 (
    item_uid       TEXT NOT NULL,
    stack_uid      TEXT NOT NULL,
    stability      REAL NOT NULL,
    difficulty     REAL NOT NULL,
    due_ms         INTEGER NOT NULL,
    last_review_ms INTEGER NOT NULL,
    PRIMARY KEY (item_uid, stack_uid)
) WITHOUT ROWID;
INSERT INTO mem_state_v21
    SELECT m.item_uid,
           COALESCE((SELECT s.uid FROM item i JOIN stack_identity s ON s.folder = i.stack_path
                     WHERE i.uid = m.item_uid), '00000000000000000000000000'),
           m.stability, m.difficulty, m.due_ms, m.last_review_ms
    FROM mem_state m;
DROP TABLE mem_state;
ALTER TABLE mem_state_v21 RENAME TO mem_state;
CREATE INDEX mem_state_due ON mem_state(due_ms);

CREATE TABLE sm2_state_v21 (
    item_uid       TEXT NOT NULL,
    stack_uid      TEXT NOT NULL,
    ease           REAL NOT NULL,
    repetitions    INTEGER NOT NULL,
    interval_days  INTEGER NOT NULL,
    due_ms         INTEGER NOT NULL,
    last_review_ms INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (item_uid, stack_uid)
) WITHOUT ROWID;
INSERT INTO sm2_state_v21
    SELECT m.item_uid,
           COALESCE((SELECT s.uid FROM item i JOIN stack_identity s ON s.folder = i.stack_path
                     WHERE i.uid = m.item_uid), '00000000000000000000000000'),
           m.ease, m.repetitions, m.interval_days, m.due_ms, m.last_review_ms
    FROM sm2_state m;
DROP TABLE sm2_state;
ALTER TABLE sm2_state_v21 RENAME TO sm2_state;


-- What the screens that show one value per item read until they know the stack (TASK-0219): the primary stack's row.
CREATE VIEW mem_primary AS
    SELECT m.* FROM mem_state m
    LEFT JOIN item i ON i.uid = m.item_uid
    LEFT JOIN stack_identity s ON s.folder = i.stack_path
    WHERE m.stack_uid = COALESCE(s.uid, '00000000000000000000000000');
CREATE VIEW sm2_primary AS
    SELECT m.* FROM sm2_state m
    LEFT JOIN item i ON i.uid = m.item_uid
    LEFT JOIN stack_identity s ON s.folder = i.stack_path
    WHERE m.stack_uid = COALESCE(s.uid, '00000000000000000000000000');
";

/// Step 22: the reviews as the screens that count per item see them. A rating given in several stacks is written as
/// one row per stack; the first row has no `rating_id` and the others carry its id, so the view `review_once` shows
/// the rating once (the primary stack's row when it took part).
pub(crate) const V22: &str = "
ALTER TABLE review ADD COLUMN rating_id INTEGER;
CREATE VIEW review_once AS SELECT * FROM review WHERE rating_id IS NULL;
";
