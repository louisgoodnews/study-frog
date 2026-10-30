//! Priority and difficulty as items and stacks state them, and what an item gets from its stacks.

use std::collections::HashMap;

use rusqlite::Connection;
use studyfrog_core::event::{Difficulty, difficulty_from_name};
use studyfrog_core::priority::{Priority, priority_from_name};

use crate::error::{StoreError, sql_err};

/// What a stack (or an item) says about itself; `None`: nothing, ask the stack above.
pub(crate) type Own = (Option<Priority>, Option<Difficulty>);

pub(crate) fn own_of(priority: Option<String>, difficulty: Option<String>) -> Own {
    (
        priority.as_deref().and_then(priority_from_name),
        difficulty.as_deref().and_then(difficulty_from_name),
    )
}

/// The traits of each stack folder as the stack's own file states them.
pub(crate) fn stack_traits(conn: &Connection) -> Result<HashMap<String, Own>, StoreError> {
    let sql = "SELECT stack_path, priority, difficulty FROM item WHERE kind = 'stack' AND stack_path IS NOT NULL";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = HashMap::new();
    for row in rows {
        let (folder, p, d) = row.map_err(sql_err)?;
        out.insert(folder, own_of(p, d));
    }
    Ok(out)
}

/// Fill what `own` leaves open from the stack at `folder`, then from the stacks above it.
pub(crate) fn inherited(
    own: Own,
    folder: Option<&str>,
    stacks: &HashMap<String, Own>,
) -> (Priority, Option<Difficulty>) {
    let (mut priority, mut difficulty) = own;
    let mut at = folder.map(str::to_owned);
    while let Some(f) = at {
        if let Some((p, d)) = stacks.get(&f) {
            priority = priority.or(*p);
            difficulty = difficulty.or(*d);
        }
        if priority.is_some() && difficulty.is_some() {
            break;
        }
        at = f.rsplit_once('/').map(|(up, _)| up.to_owned());
    }
    (priority.unwrap_or_default(), difficulty)
}
