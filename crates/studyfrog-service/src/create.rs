//! Creating stacks and cards: render through `studyfrog-md`, write a new file (never overwriting),
//! reindex, and describe the vault again.

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_core::meta::Meta;
use studyfrog_core::time::UnixMs;
use studyfrog_md::doc::{CardDoc, Doc, StackDoc};
use studyfrog_md::error::RenderError;
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_store::poll::prime;
use studyfrog_store::reindex::reindex;
use studyfrog_store::stackmap::STACK_FILE;

use crate::session::{ServiceError, VaultInfo, VaultSession, vault_info};

/// The inputs for a fresh id: the current time and 80 random bits. Supplied by the caller so the
/// use cases stay deterministic; see [`fresh_id_source`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdSource {
    /// Milliseconds since the Unix epoch.
    pub now_ms: u64,
    /// Random bytes.
    pub random: [u8; 10],
}

/// An id source from the clock and the operating system's random bytes (`/dev/urandom`).
pub fn fresh_id_source(now_ms: u64) -> Result<IdSource, ServiceError> {
    use std::io::Read;
    let mut random = [0_u8; 10];
    fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut random))
        .map_err(|e| ServiceError::Io(format!("no randomness available: {e}")))?;
    Ok(IdSource { now_ms, random })
}

fn invalid(message: &str) -> ServiceError {
    ServiceError::Invalid(message.to_owned())
}

/// A title usable in front-matter: not blank, one line, no control characters.
fn check_title(title: &str) -> Result<(), ServiceError> {
    if title.trim().is_empty() {
        return Err(invalid("the title must not be empty"));
    }
    if title
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(invalid(
            "the title must be a single line without control characters",
        ));
    }
    Ok(())
}

/// File-name friendly form of a title: lowercase letters and digits, other runs become one dash.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    trimmed
        .chars()
        .take(60)
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

/// `base` plus `suffix`, or `base-2`, `base-3`, ... until `exists` says the name is free.
fn unique_name(base: &str, suffix: &str, exists: impl Fn(&str) -> bool) -> String {
    let base = if base.is_empty() { "item" } else { base };
    let first = format!("{base}{suffix}");
    if !exists(&first) {
        return first;
    }
    (2..)
        .map(|n| format!("{base}-{n}{suffix}"))
        .find(|name| !exists(name))
        .unwrap_or(first)
}

/// Write `text` as a new file, via a temporary file so a crash cannot leave half a file.
fn write_new(path: &Path, text: &str) -> Result<(), ServiceError> {
    let io = |e: std::io::Error| ServiceError::Io(format!("{}: {e}", path.display()));
    let dir = path.parent().ok_or_else(|| invalid("no parent folder"))?;
    fs::create_dir_all(dir).map_err(io)?;
    let temp = dir.join(format!(
        ".{}.tmp",
        path.file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
    ));
    fs::write(&temp, text).map_err(io)?;
    fs::rename(&temp, path).map_err(io)
}

fn meta(title: &str, tags: &[String], ids: IdSource) -> Meta {
    Meta {
        uid: compose_ulid(ids.now_ms, ids.random),
        title: title.to_owned(),
        tags: tags.to_vec(),
        aliases: Vec::new(),
        created: Some(UnixMs(i64::try_from(ids.now_ms).unwrap_or(i64::MAX))),
        updated: None,
    }
}

fn render_error(e: RenderError) -> ServiceError {
    match e {
        RenderError::AmbiguousSeparator => {
            invalid("the front must not contain a line with only ???")
        }
        other => ServiceError::Io(other.to_string()),
    }
}

/// Bring the index up to date after a write and describe the vault.
pub(crate) fn refresh(session: &mut VaultSession) -> Result<VaultInfo, ServiceError> {
    reindex(&mut session.conn, &session.root).map_err(ServiceError::Store)?;
    session.poll = prime(&session.root).map_err(ServiceError::Store)?;
    vault_info(&session.root, &session.conn)
}

/// Create a stack: a new top-level folder named after the title, with a `_stack.md`.
pub fn create_stack(
    session: &mut VaultSession,
    title: &str,
    ids: IdSource,
) -> Result<VaultInfo, ServiceError> {
    check_title(title)?;
    let folder = unique_name(&slug(title), "", |n| session.root.join(n).exists());
    let doc = Doc::Stack(StackDoc {
        meta: meta(title, &[], ids),
        description: String::new(),
        body: String::new(),
        extra: Vec::new(),
    });
    let text = render_doc(&doc).map_err(render_error)?;
    write_new(&session.root.join(&folder).join(STACK_FILE), &text)?;
    refresh(session)
}

/// The title of a card made without one: its first non-empty line, shortened.
pub fn default_card_title(front: &str) -> String {
    let line = front
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("Card");
    line.chars().take(60).collect()
}

/// What a new card holds.
pub struct NewCard<'a> {
    /// Title; empty means "derive it from the front".
    pub title: &'a str,
    /// Prompt side.
    pub front: &'a str,
    /// Answer side.
    pub back: &'a str,
    /// Tags.
    pub tags: &'a [String],
}

/// Write one card file into the stack at `folder` without reindexing; the caller reindexes once
/// after writing any number of cards.
pub fn write_card(
    session: &VaultSession,
    folder: &str,
    card: &NewCard<'_>,
    ids: IdSource,
) -> Result<(), ServiceError> {
    if card.front.trim().is_empty() || card.back.trim().is_empty() {
        return Err(invalid("a card needs both a front and a back"));
    }
    let title = if card.title.trim().is_empty() {
        default_card_title(card.front)
    } else {
        card.title.trim().to_owned()
    };
    check_title(&title)?;
    let dir: PathBuf = session.root.join(folder);
    if !dir.join(STACK_FILE).is_file() {
        return Err(invalid("that folder is not a stack"));
    }
    let name = unique_name(&slug(&title), ".md", |n| dir.join(n).exists());
    let doc = Doc::Card(CardDoc {
        meta: meta(&title, card.tags, ids),
        reversible: false,
        front: card.front.trim_end().to_owned(),
        back: card.back.trim_end().to_owned(),
        extra: Vec::new(),
    });
    let text = render_doc(&doc).map_err(render_error)?;
    write_new(&dir.join(name), &text)
}

/// Create a card in the stack at `folder` (vault-relative). An empty title is derived from the front.
pub fn create_card(
    session: &mut VaultSession,
    folder: &str,
    title: &str,
    front: &str,
    back: &str,
    ids: IdSource,
) -> Result<VaultInfo, ServiceError> {
    let card = NewCard {
        title,
        front,
        back,
        tags: &[],
    };
    write_card(session, folder, &card, ids)?;
    refresh(session)
}
