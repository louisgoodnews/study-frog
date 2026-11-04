//! Filtering and date lookup utilities for diary entries.

use studyfrog_core::civil::{Date, date_to_days};
use studyfrog_core::diary::rating_standard_score;
use studyfrog_store::diary::StoredDiaryEntry;

/// Filter entries by a specific civil date.
pub fn entries_for_date(entries: &[StoredDiaryEntry], date: Date) -> Vec<&StoredDiaryEntry> {
    entries.iter().filter(|e| e.date == date).collect()
}

/// Whether any entry exists on `date`.
pub fn has_entry_for_date(entries: &[StoredDiaryEntry], date: Date) -> bool {
    entries.iter().any(|e| e.date == date)
}

/// Filter entries matching search text, stack, and rating criteria.
pub fn filter_entries<'a>(
    entries: &'a [StoredDiaryEntry],
    search: &str,
    stack: Option<&str>,
    rating_scale: Option<&str>,
    min_rating: Option<u8>,
) -> Vec<&'a StoredDiaryEntry> {
    let search_lower = search.trim().to_lowercase();
    let mut out: Vec<&'a StoredDiaryEntry> = entries
        .iter()
        .filter(|e| {
            if !search_lower.is_empty() {
                let matches_title = e.title.to_lowercase().contains(&search_lower);
                let matches_content = e.content.to_lowercase().contains(&search_lower);
                let matches_tags = e
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&search_lower));
                if !matches_title && !matches_content && !matches_tags {
                    return false;
                }
            }
            if let Some(target_stack) = stack {
                let s_lower = target_stack.to_lowercase();
                if !e.stacks.iter().any(|s| s.to_lowercase().contains(&s_lower)) {
                    return false;
                }
            }
            if let (Some(scale), Some(min_score)) = (rating_scale, min_rating) {
                let score = e
                    .ratings
                    .get(scale)
                    .and_then(rating_standard_score)
                    .unwrap_or(0);
                if score < min_score {
                    return false;
                }
            }
            true
        })
        .collect();
    out.sort_by_key(|e| std::cmp::Reverse(date_to_days(e.date)));
    out
}
