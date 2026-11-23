//! Context budgeter for lecture prompts (STORY-0209, TASK-0289). Small local models degrade when the
//! prompt grows, so everything sent is packed into one character budget. Pure.
//!
//! Priority and caps, in order:
//! 1. The rolling summary takes at most a quarter of the budget, cut at a character boundary.
//! 2. The recent turns take at most half of the budget. The newest turns are kept first, and packing
//!    stops at the first turn that does not fit, so the kept turns stay one contiguous tail.
//! 3. The syllabus excerpts take whatever is left, in the order given. The last one that does not fit
//!    is cut short; the ones after it are dropped.
//!
//! Characters are the unit because the tokenizer is not known here. [`chars_for_tokens`] converts a
//! token limit using a conservative ratio.

use crate::lecture_prompt::Line;

/// Characters per token, deliberately low, so a character budget also fits a token limit for
/// typical English text and code.
pub const CHARS_PER_TOKEN: usize = 3;

/// Characters that a token limit allows, at [`CHARS_PER_TOKEN`].
pub fn chars_for_tokens(tokens: usize) -> usize {
    tokens.saturating_mul(CHARS_PER_TOKEN)
}

/// What fits in the budget. Every borrowed piece is a slice of the input, so packing copies nothing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Packed<'a> {
    /// The rolling summary, possibly cut short.
    pub summary: &'a str,
    /// Syllabus excerpts that fit, in the order given. The last may be cut short.
    pub excerpts: Vec<&'a str>,
    /// Recent turns that fit, in chronological order.
    pub turns: Vec<Line<'a>>,
    /// Characters used in total. Never more than the budget.
    pub used_chars: usize,
}

/// Packs the summary, the syllabus excerpts and the recent turns into `budget_chars` characters.
/// `turns` is in chronological order, oldest first. A zero budget yields an empty packing.
pub fn pack<'a>(
    budget_chars: usize,
    summary: &'a str,
    excerpts: &[&'a str],
    turns: &[Line<'a>],
) -> Packed<'a> {
    let summary_cap = budget_chars / 4;
    let (summary_part, summary_used) = take_prefix(summary, summary_cap);
    let turns_cap = budget_chars / 2;
    let (kept_turns, turns_used) = pack_turns_newest_first(turns, turns_cap);
    let excerpt_room = budget_chars - summary_used - turns_used;
    let (kept_excerpts, excerpts_used) = pack_excerpts(excerpts, excerpt_room);
    Packed {
        summary: summary_part,
        excerpts: kept_excerpts,
        turns: kept_turns,
        used_chars: summary_used + turns_used + excerpts_used,
    }
}

/// Characters a turn costs once rendered as `speaker: text` on its own line.
pub fn line_chars(line: &Line<'_>) -> usize {
    line.speaker.chars().count() + line.text.chars().count() + 3
}

/// The longest prefix of `text` of at most `max` characters, and its character count.
fn take_prefix(text: &str, max: usize) -> (&str, usize) {
    let total = text.chars().count();
    if total <= max {
        return (text, total);
    }
    let cut = text.char_indices().nth(max).map_or(text.len(), |(i, _)| i);
    (&text[..cut], max)
}

fn pack_turns_newest_first<'a>(turns: &[Line<'a>], cap: usize) -> (Vec<Line<'a>>, usize) {
    let mut left = cap;
    let mut kept: Vec<Line<'a>> = Vec::new();
    for line in turns.iter().rev() {
        let cost = line_chars(line);
        if cost > left {
            break;
        }
        left -= cost;
        kept.push(*line);
    }
    kept.reverse();
    (kept, cap - left)
}

fn pack_excerpts<'a>(excerpts: &[&'a str], room: usize) -> (Vec<&'a str>, usize) {
    let mut left = room;
    let mut kept: Vec<&'a str> = Vec::new();
    for excerpt in excerpts {
        if left == 0 {
            break;
        }
        let (part, used) = take_prefix(excerpt, left);
        if !part.is_empty() {
            kept.push(part);
        }
        left -= used;
        if used < excerpt.chars().count() {
            break;
        }
    }
    (kept, room - left)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn take_prefix_cuts_on_a_character_boundary() {
        let (part, used) = take_prefix("héllo", 2);
        assert_eq!(part, "hé");
        assert_eq!(used, 2);
    }
}
