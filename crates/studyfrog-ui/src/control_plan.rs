//! What each control command does, as the messages a user's clicks would send. Pure: a command and
//! the state give phases; the driver applies one, lets the background work finish, then the next.
use crate::msg::MsgBrowse;
use crate::msg::MsgEditor;
use crate::msg::MsgRehearsal;
use crate::msg::MsgVault;

use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::bulk::BulkEdit;
use studyfrog_service::create_more::NewKind;

use crate::chrome::ChromeMsg;
use crate::control_cmd::{Command, EditTarget, ItemCmd, NewItemArgs, RehearseCmd, StackCmd};
use crate::history::HistoryMsg;
use crate::library_bulk::BulkMsg;
use crate::library_ui::LibraryMsg;
use crate::modal_types::{Field, ModalMsg};
use crate::msg::{EditAction, Msg};
use crate::state::{Screen, Selection, State};
use crate::transfer_types::Tab;
use crate::transfer_ui::TransferMsg;

pub mod calendar;
pub mod files;
pub mod homework;
pub mod panels;
pub mod screens;
pub mod search;
mod stacks;
use stacks::{rehearse, stack};

/// Phases of messages; background work settles between phases.
pub type Plan = Vec<Vec<Msg>>;

/// The item `name` (a ULID or a key such as `FLASHCARD_12`) stands for. Keys are looked up in the
/// library list, which is loaded once the library was shown (`nav library` or `query`).
pub fn resolve_item(state: &State, name: &str) -> Result<Uid, String> {
    if let Some(uid) = studyfrog_md::ulid::parse_ulid(name) {
        return Ok(uid);
    }
    let by_key = state.library.items.iter().find(|i| {
        i.row
            .key
            .as_deref()
            .is_some_and(|k| k.eq_ignore_ascii_case(name))
    });
    let stack = state.vault.as_ref().and_then(|v| {
        v.overview.stacks.iter().find(|s| {
            s.key
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case(name))
        })
    });
    by_key
        .map(|i| i.uid)
        .or(stack.map(|s| s.uid))
        .ok_or_else(|| {
            format!(
                "no item {name}: give a ULID, or a key once the library was loaded (nav library)"
            )
        })
}

/// The stack in `folder`, which must exist.
pub fn resolve_stack(state: &State, folder: &str) -> Result<Uid, String> {
    let vault = state.vault.as_ref().ok_or("no vault is open")?;
    let folder = folder.trim_matches('/');
    vault
        .overview
        .stacks
        .iter()
        .find(|s| s.folder == folder)
        .map(|s| s.uid)
        .ok_or_else(|| format!("no stack in folder {folder}"))
}

fn create(state: &State, a: &NewItemArgs) -> Result<Plan, String> {
    if let Some(folder) = &a.stack {
        resolve_stack(state, folder)?;
    }
    let m = |x| Msg::Modal(x);
    let mut msgs = vec![
        m(ModalMsg::OpenCreate),
        m(ModalMsg::SetKind(a.kind)),
        m(ModalMsg::SetParent(a.stack.clone())),
        m(ModalMsg::Edit(Field::Title, a.title.clone())),
    ];
    if a.kind != NewKind::Stack {
        msgs.push(m(ModalMsg::Edit(Field::Text, a.text.clone())));
        msgs.push(m(ModalMsg::Edit(Field::Answer, a.answer.clone())));
        msgs.push(m(ModalMsg::Edit(Field::Tags, a.tags.clone())));
    }
    msgs.push(m(ModalMsg::Submit));
    Ok(vec![msgs])
}

fn edit_action(target: &EditTarget, value: &str) -> EditAction {
    match target {
        EditTarget::Title => EditAction::Title(value.to_owned()),
        EditTarget::Tags => EditAction::Tags(value.to_owned()),
        EditTarget::Field(i) => EditAction::Field(*i, value.to_owned()),
    }
}

fn on_page(uid: Uid, then: Vec<EditAction>) -> Plan {
    let mut second: Vec<Msg> = then
        .into_iter()
        .map(|e| Msg::Editor(MsgEditor::Edit(e)))
        .collect();
    second.push(Msg::Editor(MsgEditor::SaveClicked));
    vec![vec![Msg::Browse(MsgBrowse::OpenItem(uid))], second]
}

fn item(state: &State, cmd: &ItemCmd) -> Result<Plan, String> {
    let uid = |name: &str| resolve_item(state, name);
    Ok(match cmd {
        ItemCmd::Create(a) => return create(state, a),
        ItemCmd::Open(name) => vec![vec![Msg::Browse(MsgBrowse::OpenItem(uid(name)?))]],
        ItemCmd::Edit(name, target, value) => on_page(uid(name)?, vec![edit_action(target, value)]),
        ItemCmd::Color(name, color) => on_page(uid(name)?, vec![EditAction::Color(color.clone())]),
        ItemCmd::Delete(name) => vec![
            vec![Msg::Browse(MsgBrowse::OpenItem(uid(name)?))],
            vec![
                Msg::Editor(MsgEditor::Edit(EditAction::AskDelete(true))),
                Msg::Editor(MsgEditor::Edit(EditAction::ConfirmDelete)),
            ],
        ],
        ItemCmd::Move(name, folder) => {
            resolve_stack(state, folder)?;
            let edit = BulkEdit::MoveTo(folder.trim_matches('/').to_owned());
            vec![vec![Msg::Library(LibraryMsg::Bulk(BulkMsg::MoveOne(
                uid(name)?,
                edit,
            )))]]
        }
    })
}

fn open_stack(folder: &str) -> Vec<Msg> {
    vec![
        Msg::Vault(MsgVault::Navigate(Screen::Library)),
        Msg::Browse(MsgBrowse::Select(Selection::Stack(
            folder.trim_matches('/').to_owned(),
        ))),
    ]
}

fn transfer(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

/// The plan for `cmd`; commands that only read (`state`, `logs`, ...) have an empty plan.
pub fn plan_for(state: &State, cmd: &Command) -> Result<Plan, String> {
    let one = |m: Msg| Ok(vec![vec![m]]);
    match cmd {
        Command::Nav(screen) => one(Msg::Vault(MsgVault::Navigate(*screen))),
        Command::Back => one(Msg::Chrome(ChromeMsg::History(HistoryMsg::Back))),
        Command::Forward => one(Msg::Chrome(ChromeMsg::History(HistoryMsg::Forward))),
        Command::VaultOpen(path) => one(Msg::Vault(MsgVault::OpenRecent(path.clone()))),
        Command::VaultClose => one(Msg::Vault(MsgVault::CloseVault)),
        Command::Item(c) => item(state, c),
        Command::Stack(c) => stack(state, c),
        Command::Homework(c) => homework::plan(state, c),
        Command::Appearance(m) => one(Msg::Appearance(*m)),
        Command::Rehearse(c) => rehearse(state, c),
        Command::Query(text) => Ok(vec![vec![
            Msg::Vault(MsgVault::Navigate(Screen::Library)),
            Msg::Browse(MsgBrowse::Select(Selection::Nothing)),
            Msg::Library(LibraryMsg::Search(crate::library_search::SearchMsg::Mode(
                true,
            ))),
            Msg::Library(LibraryMsg::Search(crate::library_search::SearchMsg::Query(
                text.clone(),
            ))),
        ]]),
        Command::Search(text, focus) => search::plan(text, *focus),
        Command::Export(a) => Ok(vec![vec![
            transfer(TransferMsg::Open(Tab::Export, a.stack.clone())),
            transfer(TransferMsg::Format(a.format)),
            transfer(TransferMsg::Destination(a.dest.clone())),
            transfer(TransferMsg::Mode(a.scheduling)),
            transfer(TransferMsg::Export),
        ]]),
        Command::Import {
            path,
            stack,
            root,
            new_stack,
        } => files::import(path, stack, *root, new_stack),
        Command::PdfFind(file, text) => files::pdf_find(file, text),
        Command::Calendar(c) => calendar::plan(c),
        Command::Screens(c) => panels::plan(state, c),
        Command::Dismiss => one(Msg::Vault(MsgVault::DismissNotice)),
        Command::CloseModal => Ok(vec![vec![
            Msg::Modal(ModalMsg::Close),
            Msg::Modal(ModalMsg::Discard),
        ]]),
        other => screens::plan(other),
    }
}
