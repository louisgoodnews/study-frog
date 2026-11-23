//! Counts with their nouns: "1 day", "2 days", "1 page". Pure.

use std::fmt::Display;

/// The noun for a count: the singular for exactly one, the plural otherwise (and for zero).
pub fn noun<'a, N: Copy + PartialEq + From<u8>>(n: N, one: &'a str, many: &'a str) -> &'a str {
    if n == N::from(1) { one } else { many }
}

/// The count and its noun: "1 page", "3 pages".
pub fn counted<N: Copy + Display + PartialEq + From<u8>>(n: N, one: &str, many: &str) -> String {
    let word = noun(n, one, many);
    format!("{n} {word}")
}

#[cfg(test)]
mod tests {
    use super::{counted, noun};

    #[test]
    fn one_takes_the_singular_and_every_other_count_the_plural() {
        assert_eq!(counted(1_u32, "day", "days"), "1 day");
        assert_eq!(counted(0_usize, "page", "pages"), "0 pages");
        assert_eq!(counted(2_i64, "day", "days"), "2 days");
        assert_eq!(noun(1_i64, "run", "runs"), "run");
        assert_eq!(noun(3_usize, "item", "items"), "items");
    }
}
