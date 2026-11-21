//! The library part of the FrogScript reference: the built-in functions and the vault objects. Checked
//! against the standard library by `tests/wiki.rs`.

use crate::wiki::{Category, Topic, topic_entry as t};

const B: Category = Category::BuiltinFunctions;
const V: Category = Category::VaultObjects;

/// The built-in function and vault object entries.
pub const LIBRARY: &[Topic] = &[
    t!(
        "print",
        B,
        "print(<values>)",
        "Nothing",
        "Writes its values to the log as standard output.",
        "print(\"hello\", 1)"
    ),
    t!(
        "log_info",
        B,
        "log_info(<text>)",
        "Nothing",
        "Writes an information line to the log.",
        "log_info(\"Inspecting items\")"
    ),
    t!(
        "log_warn",
        B,
        "log_warn(<text>)",
        "Nothing",
        "Writes a warning to the log.",
        "log_warn(\"No cards due\")"
    ),
    t!(
        "log_error",
        B,
        "log_error(<text>)",
        "Nothing",
        "Writes an error to the log.",
        "log_error(\"Stack missing\")"
    ),
    t!(
        "len",
        B,
        "len(<collection or text>)",
        "int",
        "The number of items in a collection, of entries in a dictionary, or of bytes in a text.",
        "log_info(len(cards))"
    ),
    t!(
        "now",
        B,
        "now",
        "int",
        "The current time in milliseconds since 1 January 1970.",
        "started = now"
    ),
    t!(
        "today",
        B,
        "today",
        "int",
        "The current day as a number of days since 1 January 1970.",
        "due = today + 3"
    ),
    t!(
        "days_ago",
        B,
        "days_ago(<n>)",
        "int",
        "The day n days before today, as a day number like today.",
        "from vault | where due < days_ago(0)"
    ),
    t!(
        "tag",
        B,
        "tag(<name>)",
        "bool",
        "True when the current item has this tag. Used inside a where condition.",
        "where tag(\"exam\")"
    ),
    t!(
        "type",
        B,
        "type(<kind>)",
        "bool",
        "True when the current item is of this kind (note, card or question).",
        "where type(\"card\")"
    ),
    t!(
        "difficulty",
        B,
        "difficulty(<level>)",
        "bool",
        "True when the current item has this difficulty, ignoring case.",
        "where difficulty(\"hard\")"
    ),
    t!(
        "priority",
        B,
        "priority(<level>)",
        "bool",
        "True when the current item has this priority, ignoring case.",
        "where priority(\"high\")"
    ),
    t!(
        "vault",
        V,
        "vault.stacks | vault.has_stack(name) | vault.stack(name) | vault.create_card(...) | vault.tags",
        "List or bool",
        "The vault: its stacks, whether a stack exists, one stack, a new card in a stack, and the tags in use.",
        "log_info(vault.has_stack(\"French\"))"
    ),
    t!(
        "calendar",
        V,
        "calendar.schedule_study(title, date, start, duration, stack) | calendar.upcoming_exams(days=30)",
        "Nothing or List",
        "Plans a study session, and lists the exams in the next days.",
        "calendar.upcoming_exams(7)"
    ),
    t!(
        "timetable",
        V,
        "timetable.add_entry(course, weekday, start, duration, room, stack)",
        "Nothing",
        "Adds a weekly class to the timetable.",
        "timetable.add_entry(\"Biology\", \"Mon\", \"09:00\", 90, \"A1\", \"bio\")"
    ),
    t!(
        "diary",
        V,
        "diary.entry(title, stack, duration_min, ratings, body)",
        "Nothing",
        "Writes a diary entry for a study session.",
        "diary.entry(\"Session\", \"French\", 30, {}, \"Went well\")"
    ),
    t!(
        "run",
        V,
        "run.accuracy | run.item_count | run.duration_min | run.median_latency_ms | run.stack_name",
        "float or int or text",
        "The rehearsal run that fired the script: its accuracy, item count, length, median answer time and stack. Only set during a run.",
        "log_info(run.accuracy)"
    ),
];
