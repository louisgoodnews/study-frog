//! Group partitioning for group tasks (STORY-0210, TASK-0291): splits the students into groups of a
//! chosen size. Pure.

/// Splits `members` (participant indices, in order) into groups of `size`. A group never has fewer
/// than two members: a lone remainder joins the last group. Returns no groups when `size` is below two
/// or there are fewer than two members.
pub fn partition_groups(members: &[usize], size: usize) -> Vec<Vec<usize>> {
    if size < 2 || members.len() < 2 {
        return Vec::new();
    }
    let mut groups: Vec<Vec<usize>> = members.chunks(size).map(<[usize]>::to_vec).collect();
    if groups.len() > 1 && groups.last().is_some_and(|g| g.len() < 2) {
        let lone = groups.pop().unwrap_or_default();
        if let Some(last) = groups.last_mut() {
            last.extend(lone);
        }
    }
    groups
}
