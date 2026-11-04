//! The review history by (item, stack): what each stack remembers of an item (ADR 0011).

use std::collections::HashMap;

use rusqlite::Connection;
use studyfrog_core::review::{Review, rating_from_value};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};
use crate::reviews::TimedReview;

fn uid_of(text: &str) -> Result<Uid, StoreError> {
    parse_ulid(text).ok_or_else(|| StoreError::Corrupt(format!("ulid: {text:?}")))
}

/// The reviews of each (item, stack).
pub type ByStack = HashMap<(Uid, Uid), Vec<TimedReview>>;

type Raw = (String, String, u8, (i64, f32), Option<(u32, u32)>);

/// The reviews of every (item, stack), oldest first, each with its timing when it was recorded.
pub fn load_all_reviews_by_stack(conn: &Connection) -> Result<ByStack, StoreError> {
    let sql = "SELECT item_uid, stack_uid, rating, at_ms, elapsed_days, answer_ms, expected_ms \
               FROM review ORDER BY item_uid, stack_uid, at_ms, id";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            let timing = r.get::<_, Option<u32>>(5)?.zip(r.get::<_, Option<u32>>(6)?);
            let when = (r.get::<_, i64>(3)?, r.get::<_, f32>(4)?);
            Ok::<Raw, _>((r.get(0)?, r.get(1)?, r.get(2)?, when, timing))
        })
        .map_err(sql_err)?;
    let mut out = ByStack::new();
    for row in rows {
        let (item, stack, rating, (at, elapsed_days), timing) = row.map_err(sql_err)?;
        let (item, stack) = (uid_of(&item)?, uid_of(&stack)?);
        let rating = rating_from_value(rating)
            .ok_or_else(|| StoreError::Corrupt(format!("rating: {rating}")))?;
        let review = Review {
            item,
            rating,
            at: UnixMs(at),
            elapsed_days,
        };
        out.entry((item, stack)).or_default().push((review, timing));
    }
    Ok(out)
}
