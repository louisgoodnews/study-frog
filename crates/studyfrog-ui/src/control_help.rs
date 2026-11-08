//! The command table of the control protocol: every command, what it becomes in the application and
//! whether it writes. `help` answers with it, so an agent can discover the protocol.

use serde_json::{Value, json};

/// One row: usage, what it does in the application (messages or effects), class.
pub type Row = (&'static str, &'static str, &'static str);

/// Every command.
pub const COMMANDS: &[Row] = &[
    (
        "state",
        "snapshot of State: screen, vault, selection, item page, dialog, rehearsal, library, notices",
        "read",
    ),
    ("help", "this table", "read"),
    (
        "nav <screen>",
        "Msg::Navigate(screen): home folders items|library rehearse calendar board reports diary achievements graph assistant prompts settings help welcome",
        "read",
    ),
    (
        "back | forward",
        "Msg::Chrome(History(Back|Forward))",
        "read",
    ),
    (
        "vault open <folder> | vault close",
        "Msg::OpenRecent(folder) | Msg::CloseVault",
        "write",
    ),
    (
        "item create <note|card|question> <title> [--text T] [--answer A] [--tags T] [--stack F]",
        "Modal: OpenCreate, SetKind, SetParent, Edit(fields), Submit -> Effect::CreateItem",
        "write",
    ),
    (
        "item open <ULID|KEY>",
        "Msg::OpenItem -> Effect::LoadDocument",
        "read",
    ),
    (
        "item edit <item> <title|tags|front|back|body|prompt|reference|fieldN> <text>",
        "OpenItem; Edit(Title|Tags|Field); SaveClicked -> Effect::SaveDocument",
        "write",
    ),
    (
        "item color <item> <word|#RRGGBB|none>",
        "OpenItem; Edit(Color); SaveClicked",
        "write",
    ),
    (
        "item move <item> <stack folder>",
        "Library Bulk MoveOne(MoveTo) -> bulk effect (undoable)",
        "write",
    ),
    (
        "item delete <item>",
        "OpenItem; Edit(AskDelete, ConfirmDelete) -> delete effect",
        "destructive",
    ),
    (
        "stack create <title> [--parent F]",
        "Modal: OpenCreate, SetKind(Stack), SetParent, Edit(Title), Submit",
        "write",
    ),
    (
        "stack open <folder> | stack settings <folder>",
        "Navigate(Library), Select(Stack(folder)); settings also lists scheduler and limits",
        "read",
    ),
    (
        "stack move <folder> [<into folder>]",
        "Library Stack MoveStack(src, dst)",
        "write",
    ),
    (
        "rehearse setup [--preset spaced|all|shuffled|hardest|stalest] [--mode flip|answer|exam] [--limit N] [--stack F]...",
        "Navigate(Rehearse), Setup(Preset|Mode|Limit|Stack) -> Effect::PreviewQueue",
        "read",
    ),
    (
        "rehearse start",
        "Msg::StartRunClicked -> Effect::StartRun",
        "write",
    ),
    (
        "rehearse flip|skip|pause|resume|stop",
        "Msg::RunAction(Flip|Skip|Pause|Resume|Abort)",
        "write",
    ),
    (
        "rehearse answer <text>",
        "AnswerEdited, RunAction(SubmitAnswer) -> Effect::ScoreAnswer",
        "write",
    ),
    (
        "rehearse rate again|hard|good|easy",
        "RunAction(Rate) -> Effect::RecordStep",
        "write",
    ),
    (
        "rehearse mark easy|medium|difficult",
        "RunAction(Mark)",
        "write",
    ),
    ("rehearse leave", "Msg::LeaveRun", "read"),
    (
        "rehearse type <text> | rehearse submit",
        "AnswerEdited; RunAction(SubmitAnswer(field)); in an exam the answer is locked and the next item follows",
        "write",
    ),
    (
        "rehearse finish",
        "Msg::LeaveRun once the exam is over (state: rehearse.run.exam.result)",
        "read",
    ),
    (
        "query <expr>",
        "Navigate(Library), Select(Nothing), Search(Mode(query), Query(expr)); answers the matching rows",
        "read",
    ),
    (
        "search <text>",
        "Chrome SearchEdited -> Effect::SearchItems, PDF search",
        "read",
    ),
    (
        "export <json|toml|ron|csv|markdown> <folder> [--stack F] [--scheduling]",
        "Transfer: Open(Export), Format, Destination, Mode, Export",
        "write",
    ),
    (
        "import <file> [--stack F]",
        "Transfer: Open(Import), Path, Preview; then Import",
        "write",
    ),
    (
        "screenshot <file.png>",
        "ViewportCommand::Screenshot of the window (attached window only)",
        "write",
    ),
    (
        "logs [--tail N] [--follow] [--level info|warn|error]",
        "the console.jsonl entries; --follow streams new ones as {\"event\":\"log\"} lines",
        "read",
    ),
    ("notices", "the recent notifications", "read"),
    (
        "dismiss | close",
        "Msg::DismissNotice | Modal Close, Discard (what was written in a dialog is lost)",
        "read",
    ),
    ("settle", "wait until background work is done", "read"),
];

/// The table as JSON.
pub fn help_json() -> Value {
    let rows: Vec<Value> = COMMANDS
        .iter()
        .map(|(usage, does, class)| json!({"usage": usage, "does": does, "class": class}))
        .collect();
    json!({
        "commands": rows,
        "request": "a line of words, or JSON {\"id\": any, \"cmd\": \"nav\", \"args\": [\"home\"]}",
        "answer": "{\"id\": ..., \"ok\": true, \"result\": ...} or {\"id\": ..., \"ok\": false, \"error\": \"...\"}",
    })
}
