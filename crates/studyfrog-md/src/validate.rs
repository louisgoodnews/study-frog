//! Checks that span several files.

use std::collections::HashMap;

use studyfrog_core::uid::Uid;

/// Ids used by more than one entry, with every key that uses them, sorted by id.
/// Keys are typically file paths, so each offending file can be reported.
pub fn find_duplicate_ids<K>(entries: &[(K, Uid)]) -> Vec<(Uid, Vec<&K>)> {
    let mut by_id: HashMap<Uid, Vec<&K>> = HashMap::with_capacity(entries.len());
    for (key, uid) in entries {
        by_id.entry(*uid).or_default().push(key);
    }
    let mut duplicates: Vec<_> = by_id
        .into_iter()
        .filter(|(_, keys)| keys.len() > 1)
        .collect();
    duplicates.sort_by_key(|(uid, _)| *uid);
    duplicates
}
