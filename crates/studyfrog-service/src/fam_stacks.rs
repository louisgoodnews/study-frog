//! The familiarity of each stack, for the stack view.

use studyfrog_core::time::UnixMs;
use studyfrog_store::error::StoreError;
use studyfrog_store::overview::load_overview;

use crate::fam_report::{Scored, group, in_scored, score_all};

/// The familiarity of each stack in percent, by folder (`None`: no item has a history).
pub fn stack_percents(
    conn: &rusqlite::Connection,
    now: UnixMs,
) -> Result<std::collections::HashMap<String, Option<u8>>, StoreError> {
    let stacks = load_overview(conn)?.stacks;
    let all = score_all(conn, &stacks, now)?;
    Ok(stacks
        .iter()
        .map(|st| {
            let members: Vec<&Scored> = all.iter().filter(|s| in_scored(s, &st.folder)).collect();
            (st.folder.clone(), group(st.title.clone(), &members).percent)
        })
        .collect())
}
