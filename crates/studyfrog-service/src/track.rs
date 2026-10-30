//! What a new item leaves behind: a card on the board and, for a stack with a due date, a deadline
//! on the calendar. Best effort by design: the item itself already exists, so a failure here is
//! reported to the caller rather than undoing the file.

use studyfrog_board::ops::BoardOp;
use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session, validate_session};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_store::sessions::save_session;

use crate::create::IdSource;
use crate::create_more::{Created, NewItem, NewKind, create_item};
use crate::kanban::{apply_board_op, board_or_default};
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// When a new stack is due and when to remind about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueDate {
    /// The local date.
    pub date: Date,
    /// Local minute of the day it is due.
    pub minute: u16,
    /// Lead times of the reminders, in minutes.
    pub reminders: Vec<u32>,
}

/// Default time of day a due date falls on: 09:00.
pub const DEFAULT_DUE_MINUTE: u16 = 540;

fn kind_word(kind: NewKind) -> &'static str {
    match kind {
        NewKind::Stack => "Stack",
        NewKind::Note => "Note",
        NewKind::Card => "Card",
        NewKind::Question => "Question",
    }
}

fn stack_uid(info: &VaultInfo, folder: &str) -> Option<Uid> {
    info.overview
        .stacks
        .iter()
        .find(|s| s.folder == folder)
        .map(|s| s.uid)
}

fn folder_of(kind: NewKind, created: &Created, parent: Option<&str>) -> Option<String> {
    match (kind, parent) {
        (NewKind::Stack, _) => created.path.strip_suffix("/_stack.md").map(str::to_owned),
        (_, parent) => parent.map(str::to_owned),
    }
}

/// Put a card for the new item on the first column of the board.
pub fn add_to_board(
    session: &mut VaultSession,
    info: &VaultInfo,
    (kind, created, parent): (NewKind, &Created, Option<&str>),
    ids: IdSource,
) -> Result<(), ServiceError> {
    let board = board_or_default(session, ids)?;
    let Some(first) = board.columns.first() else {
        return Ok(());
    };
    let title = format!("{}: {}", kind_word(kind), created.title);
    let board = apply_board_op(
        session,
        &BoardOp::AddCard {
            column: first.id,
            title: title.clone(),
        },
        ids,
    )?;
    let stack = folder_of(kind, created, parent).and_then(|f| stack_uid(info, &f));
    let card = board.columns[0].cards.last().filter(|c| c.title == title);
    match (card, stack) {
        (Some(c), Some(stack)) => apply_board_op(
            session,
            &BoardOp::Update {
                card: c.id,
                title,
                notes: c.notes.clone(),
                stack: Some(stack),
                points: c.points,
            },
            ids,
        )
        .map(|_| ()),
        _ => Ok(()),
    }
}

/// A deadline for a stack on the calendar, with its reminders.
pub fn add_deadline(
    session: &mut VaultSession,
    info: &VaultInfo,
    (created, due): (&Created, &DueDate),
    ids: IdSource,
) -> Result<(), ServiceError> {
    let stack = folder_of(NewKind::Stack, created, None).and_then(|f| stack_uid(info, &f));
    let mut random = ids.random;
    random[8] ^= 0x5a;
    let mut reminders = due.reminders.clone();
    reminders.sort_unstable();
    reminders.dedup();
    let entry = Session {
        id: compose_ulid(ids.now_ms, random),
        title: format!("Due: {}", created.title),
        stack,
        start: LocalTime {
            date: due.date,
            minute: due.minute,
        },
        duration_min: 15,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: EventKind::Deadline,
        reminders,
    };
    validate_session(&entry).map_err(ServiceError::Invalid)?;
    save_session(&mut session.conn, &entry).map_err(ServiceError::Store)
}

/// What [`create_tracked`] made.
#[derive(Debug, Clone, PartialEq)]
pub struct Tracked {
    /// Fresh view of the vault.
    pub info: VaultInfo,
    /// The new item.
    pub created: Created,
    /// What could not be added to the board or the calendar, in words for the user.
    pub warnings: Vec<String>,
}

/// Create an item, then give it a board card and, for a stack with a due date, a calendar deadline.
/// The item is kept even when the follow-ups fail; the failures come back as warnings.
pub fn create_tracked(
    session: &mut VaultSession,
    item: &NewItem<'_>,
    due: Option<&DueDate>,
    ids: IdSource,
) -> Result<Tracked, ServiceError> {
    let (info, created) = create_item(session, item, ids)?;
    let mut warnings = Vec::new();
    let board = add_to_board(session, &info, (item.kind, &created, item.parent), ids);
    if let Err(e) = board {
        warnings.push(format!("it was not added to the board: {e}"));
    }
    let deadline = match (item.kind, due) {
        (NewKind::Stack, Some(due)) => add_deadline(session, &info, (&created, due), ids),
        _ => Ok(()),
    };
    if let Err(e) = deadline {
        warnings.push(format!("the due date was not added to the calendar: {e}"));
    }
    Ok(Tracked {
        info,
        created,
        warnings,
    })
}
