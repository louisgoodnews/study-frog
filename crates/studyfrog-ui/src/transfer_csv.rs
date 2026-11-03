//! Changing how a CSV or TSV file is read in the import dialog (TASK-0208). Pure: every change asks for a new preview.

use studyfrog_service::xfer_csv_map::Mapping;

use crate::effect::Effect;
use crate::library_bulk::LibraryEffect;
use crate::state::State;
use crate::transfer_types::TransferJob;

/// What a column means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The front.
    Front,
    /// The back.
    Back,
    /// The title.
    Title,
    /// The tags.
    Tags,
    /// The stack.
    Stack,
    /// The kind.
    Kind,
}

/// A choice in the CSV settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvChange {
    /// The delimiter (`None`: detect it).
    Delimiter(Option<char>),
    /// Latin-1 or UTF-8 (`None`: detect it).
    Latin1(Option<bool>),
    /// The first row is a header (`None`: detect it).
    Header(Option<bool>),
    /// A column means this (`None`: the column is not used; only the optional roles may be unused).
    Column(Role, Option<usize>),
}

fn set(map: &mut Mapping, role: Role, at: Option<usize>) {
    match (role, at) {
        (Role::Front, Some(i)) => map.front = i,
        (Role::Back, Some(i)) => map.back = i,
        (Role::Front | Role::Back, None) => {}
        (Role::Title, _) => map.title = at,
        (Role::Tags, _) => map.tags = at,
        (Role::Stack, _) => map.stack = at,
        (Role::Kind, _) => map.kind = at,
    }
}

/// Apply the choice and ask for the preview again.
pub fn update(state: State, change: CsvChange) -> (State, Vec<Effect>) {
    let mut state = state;
    let used = state
        .transfer
        .preview
        .as_ref()
        .and_then(|p| p.csv.as_ref())
        .map(|c| c.sheet.map);
    let t = &mut state.transfer;
    match change {
        CsvChange::Delimiter(d) => (t.csv.delimiter, t.csv.map) = (d, None),
        CsvChange::Latin1(l) => t.csv.latin1 = l,
        CsvChange::Header(h) => (t.csv.header, t.csv.map) = (h, None),
        CsvChange::Column(role, at) => {
            if let Some(mut map) = t.csv.map.or(used) {
                set(&mut map, role, at);
                t.csv.map = Some(map);
            }
        }
    }
    t.busy = true;
    t.status = None;
    let job = TransferJob::Preview(t.path.trim().to_owned(), t.scope.clone(), t.csv.clone());
    (state, vec![Effect::Library(LibraryEffect::Transfer(job))])
}
