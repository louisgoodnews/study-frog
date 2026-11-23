//! The command table rows of the board, diary, reports and calendar answers (STORY-0124, EPIC-064).

use crate::control_help::Row;

/// Rows for the screens driven by the protocol.
pub const ROWS: &[Row] = &[
    (
        "calendar view | calendar period (no argument)",
        "answer only: the selection on screen (view, month, selected day and its sessions)",
        "read",
    ),
    (
        "board | board list",
        "Navigate(Board) -> Effect::LoadBoard; answer: every column with its cards (status = column)",
        "read",
    ),
    (
        "board column <name>",
        "Navigate(Board); answer: that column with its cards",
        "read",
    ),
    (
        "board add <title> [--column <name>]",
        "Navigate(Board); Board NewTitle, then Add (first column) or AddTo(column) -> Effect::BoardCommand(AddCard)",
        "write",
    ),
    (
        "diary | diary list [<date> [<date>]]",
        "Navigate(Diary) -> Effect::Diary(LoadEntries); answer: the entries of the day or range, by day",
        "read",
    ),
    (
        "diary open <ULID>",
        "Navigate(Diary); Diary EditEntry(uid) opens the entry in the editor; answer: the entry, editing true",
        "read",
    ),
    (
        "diary add <YYYY-MM-DD> <title> [--text T] [--minutes N] [--tags T] [--stacks S]",
        "Navigate(Diary); Diary NewEntry, UpdateDraft*, SaveDraft -> Effect::Diary(SaveEntry); a refusal is an error",
        "write",
    ),
    (
        "reports | reports <tab> | reports tab <tab>",
        "Navigate(Reports); Explorer Tab(tab); answer: the tab's figures (performance, memory, familiarity, calibration, cognitive, runs, habits, leeches, workload)",
        "read",
    ),
];
