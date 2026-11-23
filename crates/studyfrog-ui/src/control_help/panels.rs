//! The command table rows of the widget tree, the screens that had only `nav` (achievements, graph,
//! assistant, settings, match, folders, PDF open) and `stack delete` (STORY-0124, EPIC-064).

use crate::control_help::Row;

/// Rows for the widget tree and the screens' commands.
pub const ROWS: &[Row] = &[
    (
        "ui | ui tree",
        "the window's AccessKit tree of the frame: every named widget (role, label, value, enabled), at most 300; a headless instance refuses",
        "read",
    ),
    (
        "ui find <label>",
        "the widgets labelled with this text (exactly, else containing it); an error when none",
        "read",
    ),
    (
        "ui click <label>",
        "queues a click on the one widget labelled so (egui's AccessKit Click action); answers after the frame that took it. A label with delete, remove or discard needs --control-allow-destructive",
        "write",
    ),
    (
        "achievements | achievements list",
        "Navigate(Achievements), Achievements(Wanted) -> Effect::CheckAchievements; answer: every entry with its progress and whether it is unlocked",
        "read",
    ),
    (
        "achievements filter all|global|stack <folder>",
        "Achievements(Filter): the list shown under a filter (a view setting, nothing is saved)",
        "read",
    ),
    (
        "graph",
        "Navigate(Graph) -> Effect::LoadGraph; answer: the vault's link graph, nodes and edges (the graph has no stack scope)",
        "read",
    ),
    (
        "assistant | assistant state",
        "Navigate(Agent); answer: tab, connections (with keys, no key text), selected model, chat, and whether a model is configured",
        "read",
    ),
    (
        "assistant send <prompt>",
        "Navigate(Agent), AgentMsg::ChatInput, ChatSend; refused with \"not configured\" when no model is connected; answer: the last reply",
        "write",
    ),
    (
        "settings | settings read",
        "Navigate(Settings); answer: the sections (tab keys and labels), the current one and slow-answer-notice",
        "read",
    ),
    (
        "settings slow-answer on|off",
        "Navigate(Settings), SettingsTabMsg::SlowAnswerNotice (saved to app.toml)",
        "write",
    ),
    (
        "match | match state",
        "answer only: the match game on screen (tiles, pairs left, mismatches, time, high score); active false when none",
        "read",
    ),
    (
        "match start",
        "Navigate(Library), Match(StartFromVault): a new game from the vault's items, on the match screen",
        "write",
    ),
    (
        "folders | folders list",
        "Navigate(Folders) -> scan; answer: the entries (path, folder, stack, depth, expanded, selected), at most 500",
        "read",
    ),
    (
        "folders open <path>",
        "Navigate(Folders), Folders(Select, ToggleExpand when collapsed); answer: the entry and its children; an error when the path is not in the vault",
        "read",
    ),
    (
        "pdf open <path>",
        "Pdf(Open(path, 0)); answer: the viewer with its page_count, query and the pages the find text was found on",
        "read",
    ),
    (
        "stack delete <folder>",
        "Folders(AskDelete) -> the delete plan, then Folders(ConfirmDelete): the confirmation the folders screen asks for; an error while the stack is still there",
        "destructive",
    ),
];
