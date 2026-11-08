//! What each control command does, as the messages a user's clicks would send. Pure: a command and
//! the state give a plan, a list of phases; the driver applies one phase, lets the background work
//! finish, then applies the next (an item must be loaded before its page can be changed).

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
    let mut second: Vec<Msg> = then.into_iter().map(Msg::Edit).collect();
    second.push(Msg::SaveClicked);
    vec![vec![Msg::OpenItem(uid)], second]
}

fn item(state: &State, cmd: &ItemCmd) -> Result<Plan, String> {
    let uid = |name: &str| resolve_item(state, name);
    Ok(match cmd {
        ItemCmd::Create(a) => return create(state, a),
        ItemCmd::Open(name) => vec![vec![Msg::OpenItem(uid(name)?)]],
        ItemCmd::Edit(name, target, value) => on_page(uid(name)?, vec![edit_action(target, value)]),
        ItemCmd::Color(name, color) => on_page(uid(name)?, vec![EditAction::Color(color.clone())]),
        ItemCmd::Delete(name) => vec![
            vec![Msg::OpenItem(uid(name)?)],
            vec![
                Msg::Edit(EditAction::AskDelete(true)),
                Msg::Edit(EditAction::ConfirmDelete),
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
        Msg::Navigate(Screen::Library),
        Msg::Select(Selection::Stack(folder.trim_matches('/').to_owned())),
    ]
}

fn stack(state: &State, cmd: &StackCmd) -> Result<Plan, String> {
    Ok(match cmd {
        StackCmd::Create(title, parent) => {
            let args = NewItemArgs {
                kind: NewKind::Stack,
                title: title.clone(),
                text: String::new(),
                answer: String::new(),
                tags: String::new(),
                stack: parent.clone(),
            };
            return create(state, &args);
        }
        StackCmd::Open(folder) | StackCmd::Settings(folder) => {
            resolve_stack(state, folder)?;
            vec![open_stack(folder)]
        }
        StackCmd::Move(folder, into) => {
            resolve_stack(state, folder)?;
            if !into.is_empty() {
                resolve_stack(state, into)?;
            }
            let m = crate::stack_view_ui::StackViewMsg::MoveStack(folder.clone(), into.clone());
            vec![vec![Msg::Library(LibraryMsg::Stack(m))]]
        }
    })
}

fn rehearse(state: &State, cmd: &RehearseCmd) -> Result<Plan, String> {
    crate::control_exam::check(state, cmd)?;
    let act = |a| vec![vec![Msg::RunAction(a)]];
    Ok(match cmd {
        RehearseCmd::Setup(args) => vec![crate::control_setup::setup_msgs(state, args)?],
        RehearseCmd::Start => vec![vec![Msg::Navigate(Screen::Rehearse), Msg::StartRunClicked]],
        RehearseCmd::Flip => act(Action::Flip),
        RehearseCmd::Answer(text) => vec![vec![
            Msg::AnswerEdited(text.clone()),
            Msg::RunAction(Action::SubmitAnswer(text.clone())),
        ]],
        RehearseCmd::Rate(r) => act(Action::Rate(*r)),
        RehearseCmd::Mark(d) => act(Action::Mark(*d)),
        RehearseCmd::Skip => act(Action::Skip),
        RehearseCmd::Pause => act(Action::Pause),
        RehearseCmd::Resume => act(Action::Resume),
        RehearseCmd::Stop => act(Action::Abort),
        RehearseCmd::Leave | RehearseCmd::Finish => vec![vec![Msg::LeaveRun]],
        RehearseCmd::Type(text) => vec![vec![Msg::AnswerEdited(text.clone())]],
        RehearseCmd::Submit => {
            let text = state
                .rehearse
                .run
                .as_ref()
                .map(|r| r.answer.clone())
                .unwrap_or_default();
            act(Action::SubmitAnswer(text))
        }
    })
}

fn transfer(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

/// The plan for `cmd`; commands that only read (`state`, `logs`, ...) have an empty plan.
pub fn plan_for(state: &State, cmd: &Command) -> Result<Plan, String> {
    let one = |m: Msg| Ok(vec![vec![m]]);
    match cmd {
        Command::Nav(screen) => one(Msg::Navigate(*screen)),
        Command::Back => one(Msg::Chrome(ChromeMsg::History(HistoryMsg::Back))),
        Command::Forward => one(Msg::Chrome(ChromeMsg::History(HistoryMsg::Forward))),
        Command::VaultOpen(path) => one(Msg::OpenRecent(path.clone())),
        Command::VaultClose => one(Msg::CloseVault),
        Command::Item(c) => item(state, c),
        Command::Stack(c) => stack(state, c),
        Command::Rehearse(c) => rehearse(state, c),
        Command::Query(text) => Ok(vec![vec![
            Msg::Navigate(Screen::Library),
            Msg::Select(Selection::Nothing),
            Msg::Library(LibraryMsg::Search(crate::library_search::SearchMsg::Mode(
                true,
            ))),
            Msg::Library(LibraryMsg::Search(crate::library_search::SearchMsg::Query(
                text.clone(),
            ))),
        ]]),
        Command::Search(text) => one(Msg::Chrome(ChromeMsg::SearchEdited(text.clone()))),
        Command::Export(a) => Ok(vec![vec![
            transfer(TransferMsg::Open(Tab::Export, a.stack.clone())),
            transfer(TransferMsg::Format(a.format)),
            transfer(TransferMsg::Destination(a.dest.clone())),
            transfer(TransferMsg::Mode(a.scheduling)),
            transfer(TransferMsg::Export),
        ]]),
        Command::Import { path, stack } => Ok(vec![
            vec![
                transfer(TransferMsg::Open(Tab::Import, stack.clone())),
                transfer(TransferMsg::Path(path.clone())),
                transfer(TransferMsg::Preview),
            ],
            vec![transfer(TransferMsg::Import)],
        ]),
        Command::Dismiss => one(Msg::DismissNotice),
        Command::CloseModal => Ok(vec![vec![
            Msg::Modal(ModalMsg::Close),
            Msg::Modal(ModalMsg::Discard),
        ]]),
        _ => Ok(Vec::new()),
    }
}
