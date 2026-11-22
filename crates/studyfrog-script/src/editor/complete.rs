//! Completion for FrogScript (STORY-0150): the names that finish the word before the cursor, and inserting one.

use super::{BUILTINS, KEYWORDS, VAULT_MEMBERS};

/// The word being typed before `cursor`: its start and the letters so far (identifiers and a `.` member chain).
fn word_before(src: &str, cursor: usize) -> (usize, &str, Option<&str>) {
    let cursor = cursor.min(src.len());
    let head = &src[..cursor];
    let start = head
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .map_or(0, |i| i + 1);
    let word = &head[start..];
    let owner = head[..start].strip_suffix('.').map(|before| {
        let s = before
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map_or(0, |i| i + 1);
        &before[s..]
    });
    (start, word, owner.filter(|o| !o.is_empty()))
}

/// The names that complete the word before `cursor`, in a stable order. After `vault.`, the vault's members; otherwise
/// the keywords, the built-ins and `vault`. Empty when nothing is being typed.
pub fn completions(src: &str, cursor: usize) -> Vec<String> {
    let (_, word, owner) = word_before(src, cursor);
    if word.is_empty() && owner.is_none() {
        return Vec::new();
    }
    let pool: Vec<&str> = match owner {
        Some("vault") => VAULT_MEMBERS.to_vec(),
        Some(_) => Vec::new(),
        None => KEYWORDS
            .iter()
            .chain(BUILTINS)
            .copied()
            .chain(["vault"])
            .collect(),
    };
    let mut out: Vec<String> = pool
        .into_iter()
        .filter(|n| n.starts_with(word) && *n != word)
        .map(str::to_owned)
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The text with `choice` put in place of the word before `cursor`, and the cursor after it.
pub fn with_completion(src: &str, cursor: usize, choice: &str) -> (String, usize) {
    let cursor = cursor.min(src.len());
    let (start, _, _) = word_before(src, cursor);
    let mut out = String::with_capacity(src.len() + choice.len());
    out.push_str(&src[..start]);
    out.push_str(choice);
    out.push_str(&src[cursor..]);
    (out, start + choice.len())
}
