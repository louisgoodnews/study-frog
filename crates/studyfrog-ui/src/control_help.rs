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
        "homework list | homework tab all|active|due-today|overdue|completed",
        "The homework items as the app last read them (JSON); the tab shown in the homework screen",
        "read",
    ),
    (
        "homework new <title> [--due YYYY-MM-DD] [--time HH:MM] [--estimate MIN] [--priority low|medium|high] [--stack F] [--checklist \"a;b\"]",
        "Homework: the dialog filled in and saved (a refusal is an error)",
        "write",
    ),
    (
        "homework advance <ULID> | homework tick <ULID> <step>",
        "Homework: advance an item's status (not started, in progress, completed); tick checklist step n (from 1)",
        "write",
    ),
    (
        "appearance theme system|dark|light | density compact|comfortable|spacious | text <percent> | accent blue|green|purple|orange|pink|teal",
        "Appearance: one setting changed and saved; the answer is the appearance now, with the spacing factor of its density",
        "write",
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
        "AnswerEdited, RunAction(SubmitAnswer) -> Effect::Answer(AnswerEffect::Score)",
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
        "search <text> | search --focus [text]",
        "Chrome SearchEdited -> Effect::SearchItems, PDF search; --focus gives the search field of the screen the keyboard and the answer names it",
        "read",
    ),
    (
        "calendar view <month|week|day|timetable>",
        "Planner SetViewMode; the calendar screen is opened first",
        "read",
    ),
    (
        "calendar period <YYYY-MM-DD>",
        "Planner Month (the month of the date) and Select(date)",
        "read",
    ),
    (
        "calendar homework <YYYY-MM-DD>",
        "Planner Select(date); the answer lists the homework due that day with its urgency",
        "read",
    ),
    (
        "calendar session add <title> --start \"YYYY-MM-DD HH:MM\" --minutes N",
        "Planner: the day and month shown, New, the form's Title, Date, Time and Duration, then Save (the same path as the form); the answer is the created session",
        "write",
    ),
    (
        "reminders due",
        "Effect::PeekReminders: the calendar and homework reminders due now, not marked shown",
        "read",
    ),
    (
        "help tooltip <name>",
        "The explanation and shortcut of a control (view::tips); the names are listed on an error",
        "read",
    ),
    (
        "export <json|toml|ron|csv|markdown> <folder> [--stack F] [--scheduling]",
        "Transfer: Open(Export), Format, Destination, Mode, Export",
        "write",
    ),
    (
        "pdf find <text> [--file F]",
        "PDF: Open(file) when given, then Query(text) in the viewer's find field; the answer lists the pages found",
        "read",
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
    (
        "subscribe [vault_opened|vault_closed|item_created|item_saved|rehearsal_finished|script_finished]...",
        "streams lifecycle events (all kinds when none named) as {\"event\":\"lifecycle\"} lines",
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
