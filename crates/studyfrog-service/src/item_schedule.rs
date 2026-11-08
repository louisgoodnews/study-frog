//! The schedule of one item in every stack that holds it, FSRS and SM-2 alike (STORY-0118): what an
//! AI client reads next to the item's Markdown. Only reads the index.

use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::error::StoreError;

/// The state one scheduler keeps for an item in one stack.
#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleState {
    /// FSRS: stability (days) and difficulty (1 to 10).
    Fsrs {
        /// Stability in days.
        stability: f32,
        /// Difficulty, 1 to 10.
        difficulty: f32,
    },
    /// SM-2: ease, repetitions and interval.
    Sm2 {
        /// Ease factor.
        ease: f32,
        /// Successful repetitions in a row.
        repetitions: u32,
        /// Interval in days.
        interval_days: u32,
    },
}

/// The schedule of an item in one stack.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemSchedule {
    /// The stack's id.
    pub stack: Uid,
    /// The stack's folder, when the stack still exists.
    pub folder: Option<String>,
    /// When the item is next due, in Unix milliseconds.
    pub due_ms: i64,
    /// When it was last reviewed, in Unix milliseconds (0: never).
    pub last_review_ms: i64,
    /// What the scheduler keeps.
    pub state: ScheduleState,
}

fn sql_err(e: rusqlite::Error) -> StoreError {
    StoreError::Sql(e.to_string())
}

type RawRow = (String, Option<String>, f64, f64, i64, i64, i64);

fn read_rows(conn: &Connection, sql: &str, uid: &str) -> Result<Vec<RawRow>, StoreError> {
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([uid], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .map_err(sql_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_err)
}

const FSRS_SQL: &str = "SELECT m.stack_uid, s.folder, m.stability, m.difficulty, 0, m.due_ms, \
     m.last_review_ms FROM mem_state m LEFT JOIN stack_identity s ON s.uid = m.stack_uid \
     WHERE m.item_uid = ?1 ORDER BY m.stack_uid";
const SM2_SQL: &str = "SELECT m.stack_uid, s.folder, m.ease, m.repetitions, m.interval_days, \
     m.due_ms, m.last_review_ms FROM sm2_state m LEFT JOIN stack_identity s ON s.uid = m.stack_uid \
     WHERE m.item_uid = ?1 ORDER BY m.stack_uid";

fn to_u32(v: f64) -> u32 {
    if v <= 0.0 {
        return 0;
    }
    // Repetitions are whole numbers stored as integers; the cast only drops a fraction there is none of.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = v.min(f64::from(u32::MAX)) as u32;
    n
}

fn schedule_of(row: RawRow, fsrs: bool) -> Option<ItemSchedule> {
    let (stack, folder, a, b, c, due_ms, last_review_ms) = row;
    #[allow(clippy::cast_possible_truncation)]
    let state = if fsrs {
        ScheduleState::Fsrs {
            stability: a as f32,
            difficulty: b as f32,
        }
    } else {
        ScheduleState::Sm2 {
            ease: a as f32,
            repetitions: to_u32(b),
            interval_days: u32::try_from(c).unwrap_or(0),
        }
    };
    Some(ItemSchedule {
        stack: studyfrog_md::ulid::parse_ulid(&stack)?,
        folder,
        due_ms,
        last_review_ms,
        state,
    })
}

/// Every schedule the index holds for `item`: FSRS ones first, then SM-2 ones, each by stack.
pub fn item_schedules(conn: &Connection, item: Uid) -> Result<Vec<ItemSchedule>, StoreError> {
    let uid = format_ulid(item);
    let mut out = Vec::new();
    for (sql, fsrs) in [(FSRS_SQL, true), (SM2_SQL, false)] {
        let rows = read_rows(conn, sql, &uid)?;
        out.extend(rows.into_iter().filter_map(|r| schedule_of(r, fsrs)));
    }
    Ok(out)
}
