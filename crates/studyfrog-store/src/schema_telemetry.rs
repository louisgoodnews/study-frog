//! Schema step 30: cognitive interaction telemetry per answer (STORY-0176).

pub(crate) const V30: &str = "
CREATE TABLE review_telemetry (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    item_uid        TEXT NOT NULL,
    reviewed_at     INTEGER NOT NULL,
    latency_ms      INTEGER NOT NULL,
    dwell_ms        INTEGER NOT NULL,
    semantic_score  REAL,
    text_complexity INTEGER NOT NULL
);
CREATE INDEX idx_telemetry_item_time ON review_telemetry(item_uid, reviewed_at);
CREATE INDEX idx_telemetry_time ON review_telemetry(reviewed_at);
";
