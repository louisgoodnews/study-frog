//! The review history: appending a review (with the timing of its answer), correcting the last rating and reading it back.

use rusqlite::{Connection, params};
use studyfrog_core::review::{Rating, Review, rating_from_value, rating_value};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};

/// Append a review to the history.
pub fn insert_review(conn: &Connection, r: &Review) -> Result<(), StoreError> {
    insert_review_timed(conn, r, None)
}

/// Append a review with the time the answer was on screen and the time it should take to read, when known.
pub fn insert_review_timed(
    conn: &Connection,
    r: &Review,
    timing: Option<(u32, u32)>,
) -> Result<(), StoreError> {
    let stack = crate::schedule::primary_stack(conn, r.item)?;
    insert_review_in(conn, r, stack, timing)
}

/// Append a review made in one stack.
pub fn insert_review_in(
    conn: &Connection,
    r: &Review,
    stack: Uid,
    timing: Option<(u32, u32)>,
) -> Result<(), StoreError> {
    insert_review_of(conn, r, (stack, None), timing).map(drop)
}

/// Append a review made in one stack as part of a rating: `first` is the id of the rating's first row, `None` for
/// the first. Returns the new row's id.
pub fn insert_review_of(
    conn: &Connection,
    r: &Review,
    (stack, first): (Uid, Option<i64>),
    timing: Option<(u32, u32)>,
) -> Result<i64, StoreError> {
    conn.prepare_cached(
        "INSERT INTO review (item_uid, rating, at_ms, elapsed_days, answer_ms, expected_ms, stack_uid, rating_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .and_then(|mut s| {
        s.execute(params![
            format_ulid(r.item),
            rating_value(r.rating),
            r.at.0,
            r.elapsed_days,
            timing.map(|t| t.0),
            timing.map(|t| t.1),
            format_ulid(stack),
            first
        ])
    })
    .map(|_| conn.last_insert_rowid())
    .map_err(sql_err)
}

/// Replace the rating of the item's latest review, in every stack it was written for. Returns false when the item
/// has no review.
pub fn set_latest_review_rating(
    conn: &Connection,
    item: Uid,
    rating: Rating,
) -> Result<bool, StoreError> {
    conn.execute(
        "UPDATE review SET rating = ?1 WHERE item_uid = ?2 AND COALESCE(rating_id, id) = \
         (SELECT id FROM review_once WHERE item_uid = ?2 ORDER BY at_ms DESC, id DESC LIMIT 1)",
        params![rating_value(rating), format_ulid(item)],
    )
    .map(|n| n > 0)
    .map_err(sql_err)
}

/// The review history of an item, oldest first.
pub fn load_reviews(conn: &Connection, item: Uid) -> Result<Vec<Review>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT rating, at_ms, elapsed_days FROM review_once WHERE item_uid = ?1 ORDER BY at_ms, id",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(item)], |r| {
            Ok((r.get::<_, u8>(0)?, r.get::<_, i64>(1)?, r.get::<_, f32>(2)?))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (rating, at, elapsed_days) = row.map_err(sql_err)?;
        let rating = rating_from_value(rating)
            .ok_or_else(|| StoreError::Corrupt(format!("rating: {rating}")))?;
        out.push(Review {
            item,
            rating,
            at: UnixMs(at),
            elapsed_days,
        });
    }
    Ok(out)
}

/// A review with its timing: the time the answer was on screen and the time it should take to read.
pub type TimedReview = (Review, Option<(u32, u32)>);

/// The reviews of an item, oldest first, each with its timing when it was recorded.
pub fn load_reviews_timed(conn: &Connection, item: Uid) -> Result<Vec<TimedReview>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT rating, at_ms, elapsed_days, answer_ms, expected_ms FROM review_once WHERE item_uid = ?1 ORDER BY at_ms, id",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(item)], |r| {
            Ok((
                r.get::<_, u8>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, f32>(2)?,
                r.get::<_, Option<u32>>(3)?,
                r.get::<_, Option<u32>>(4)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (rating, at, elapsed_days, answer, expected) = row.map_err(sql_err)?;
        let rating = rating_from_value(rating)
            .ok_or_else(|| StoreError::Corrupt(format!("rating: {rating}")))?;
        let review = Review {
            item,
            rating,
            at: UnixMs(at),
            elapsed_days,
        };
        out.push((review, answer.zip(expected)));
    }
    Ok(out)
}

/// The reviews of every item with their timing, oldest first, by item.
pub fn load_all_reviews_timed(
    conn: &Connection,
) -> Result<std::collections::HashMap<Uid, Vec<TimedReview>>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT item_uid, rating, at_ms, elapsed_days, answer_ms, expected_ms FROM review_once ORDER BY item_uid, at_ms, id",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u8>(1)?,
                (r.get::<_, i64>(2)?, r.get::<_, f32>(3)?),
                r.get::<_, Option<u32>>(4)?.zip(r.get::<_, Option<u32>>(5)?),
            ))
        })
        .map_err(sql_err)?;
    let mut out: std::collections::HashMap<Uid, Vec<TimedReview>> =
        std::collections::HashMap::new();
    for row in rows {
        let (id, rating, (at, elapsed_days), timing) = row.map_err(sql_err)?;
        let item = studyfrog_md::ulid::parse_ulid(&id)
            .ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
        let rating = rating_from_value(rating)
            .ok_or_else(|| StoreError::Corrupt(format!("rating: {rating}")))?;
        let review = Review {
            item,
            rating,
            at: UnixMs(at),
            elapsed_days,
        };
        out.entry(item).or_default().push((review, timing));
    }
    Ok(out)
}

/// The reviews an item has in one stack, oldest first, each with its timing when it was recorded.
pub fn load_reviews_timed_in(
    conn: &Connection,
    (item, stack): (Uid, Uid),
) -> Result<Vec<TimedReview>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT rating, at_ms, elapsed_days, answer_ms, expected_ms FROM review \
             WHERE item_uid = ?1 AND stack_uid = ?2 ORDER BY at_ms, id",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(item), format_ulid(stack)], |r| {
            Ok((
                (r.get::<_, u8>(0)?, r.get::<_, i64>(1)?, r.get::<_, f32>(2)?),
                r.get::<_, Option<u32>>(3)?.zip(r.get::<_, Option<u32>>(4)?),
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let ((rating, at, elapsed_days), timing) = row.map_err(sql_err)?;
        let rating = rating_from_value(rating)
            .ok_or_else(|| StoreError::Corrupt(format!("rating: {rating}")))?;
        let review = Review {
            item,
            rating,
            at: UnixMs(at),
            elapsed_days,
        };
        out.push((review, timing));
    }
    Ok(out)
}
