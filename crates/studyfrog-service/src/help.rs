//! The help pages: the supplied ones compiled into the binary and the user's own from the vault's
//! `wiki/` folder, how a link finds its page, and the search over both. Pure: no I/O here.

use studyfrog_md::links::extract_wikilinks;

pub use crate::help_pages::PAGES;

/// Where a supplied page sits in the page tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// What StudyFrog is and the first steps.
    Start,
    /// One how-to per screen.
    Screens,
    /// FAQ, glossary and the details.
    Reference,
}

/// The sections of the tree in order, with their names.
pub const SECTIONS: [(Section, &str); 3] = [
    (Section::Start, "Start"),
    (Section::Screens, "Screens"),
    (Section::Reference, "Reference"),
];

/// A page that comes with StudyFrog (read-only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelpPage {
    /// Stable id, the file name in `docs/help/` without `.md`.
    pub id: &'static str,
    /// Where it is listed.
    pub section: Section,
    /// The Markdown, starting with `# Title`.
    pub text: &'static str,
}

/// A page the user wrote, from `wiki/<file>.md` in the vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPage {
    /// The file name without `.md`.
    pub file: String,
    /// The Markdown.
    pub text: String,
}

/// Which page: a supplied one by id or a user page by file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageRef {
    /// A supplied page.
    Supplied(String),
    /// A page in `wiki/`.
    User(String),
}

/// One search result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// The page.
    pub page: PageRef,
    /// Its title.
    pub title: String,
    /// The line that matched, or the start of the page when the title matched.
    pub snippet: String,
}

/// Most search results.
pub const MAX_HITS: usize = 30;

/// The supplied page with this id.
pub fn supplied(id: &str) -> Option<&'static HelpPage> {
    PAGES.iter().find(|p| p.id == id)
}

/// The title of a page: its first `# ` heading, or `fallback` when it has none.
pub fn title_of<'a>(text: &'a str, fallback: &'a str) -> &'a str {
    text.lines()
        .find_map(|l| l.strip_prefix("# "))
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or(fallback)
}

/// The title of a user page.
pub fn user_title(page: &UserPage) -> &str {
    title_of(&page.text, &page.file)
}

/// The page a wikilink target points to: a supplied id, then a user page by file name, then any page by
/// title (case does not matter).
pub fn resolve(target: &str, user: &[UserPage]) -> Option<PageRef> {
    let t = target.trim();
    let by_id = supplied(t).map(|p| PageRef::Supplied(p.id.to_owned()));
    let by_file = || {
        user.iter()
            .find(|u| u.file.eq_ignore_ascii_case(t))
            .map(|u| PageRef::User(u.file.clone()))
    };
    let by_title = || {
        PAGES
            .iter()
            .find(|p| title_of(p.text, p.id).eq_ignore_ascii_case(t))
            .map(|p| PageRef::Supplied(p.id.to_owned()))
            .or_else(|| {
                user.iter()
                    .find(|u| user_title(u).eq_ignore_ascii_case(t))
                    .map(|u| PageRef::User(u.file.clone()))
            })
    };
    by_id.or_else(by_file).or_else(by_title)
}

/// Every (page id, target) of a supplied page whose link points to no supplied page.
pub fn broken_links() -> Vec<(&'static str, String)> {
    PAGES
        .iter()
        .flat_map(|p| {
            extract_wikilinks(p.text)
                .into_iter()
                .map(move |l| (p.id, l))
        })
        .filter(|(_, l)| supplied(l.target.trim()).is_none())
        .map(|(id, l)| (id, l.target.to_owned()))
        .collect()
}

fn snippet_of(text: &str, needle: &str) -> Option<String> {
    text.lines()
        .filter(|l| !l.starts_with("# "))
        .find(|l| l.to_lowercase().contains(needle))
        .map(|l| cut(l.trim(), 120))
}

fn cut(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let short: String = text.chars().take(max - 1).collect();
    format!("{}…", short.trim_end())
}

fn first_line(text: &str) -> String {
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("");
    cut(line, 120)
}

fn hit(page: PageRef, title: &str, text: &str, needle: &str) -> Option<(u8, Hit)> {
    let in_title = title.to_lowercase().contains(needle);
    let snippet = if in_title {
        Some(first_line(text))
    } else {
        snippet_of(text, needle)
    };
    snippet.map(|snippet| {
        let rank = u8::from(!in_title);
        let title = title.to_owned();
        (
            rank,
            Hit {
                page,
                title,
                snippet,
            },
        )
    })
}

/// The pages whose title or text contains `query` (case does not matter): title matches first, then the
/// rest, supplied before user pages within each, at most [`MAX_HITS`].
pub fn search(query: &str, user: &[UserPage]) -> Vec<Hit> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let supplied = PAGES.iter().filter_map(|p| {
        let page = PageRef::Supplied(p.id.to_owned());
        hit(page, title_of(p.text, p.id), p.text, &needle)
    });
    let own = user.iter().filter_map(|u| {
        let page = PageRef::User(u.file.clone());
        hit(page, user_title(u), &u.text, &needle)
    });
    let mut found: Vec<(u8, Hit)> = supplied.chain(own).collect();
    found.sort_by_key(|(rank, _)| *rank);
    found.into_iter().take(MAX_HITS).map(|(_, h)| h).collect()
}
