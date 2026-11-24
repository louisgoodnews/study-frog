#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Group partitioning (STORY-0210, TASK-0291): every group has two or more members, and no member
//! is lost or repeated.

use studyfrog_agent::lecture_groups::partition_groups;

fn flat(groups: &[Vec<usize>]) -> Vec<usize> {
    groups.iter().flatten().copied().collect()
}

#[test]
fn an_even_split_makes_full_groups() {
    assert_eq!(
        partition_groups(&[1, 2, 3, 4], 2),
        vec![vec![1, 2], vec![3, 4]]
    );
}

#[test]
fn a_lone_remainder_joins_the_last_group() {
    assert_eq!(partition_groups(&[1, 2, 3], 2), vec![vec![1, 2, 3]]);
    assert_eq!(
        partition_groups(&[1, 2, 3, 4, 5], 2),
        vec![vec![1, 2], vec![3, 4, 5]]
    );
}

#[test]
fn every_member_appears_once_whatever_the_size() {
    let members: Vec<usize> = (1..=11).collect();
    for size in 2..=12 {
        let groups = partition_groups(&members, size);
        assert_eq!(
            flat(&groups),
            members,
            "size {size} keeps every member once"
        );
        assert!(
            groups.iter().all(|g| g.len() >= 2),
            "size {size} has no group of one"
        );
    }
}

#[test]
fn no_groups_when_the_size_or_the_roster_is_too_small() {
    assert!(partition_groups(&[1, 2, 3], 1).is_empty());
    assert!(partition_groups(&[1], 2).is_empty());
    assert!(partition_groups(&[], 2).is_empty());
}
