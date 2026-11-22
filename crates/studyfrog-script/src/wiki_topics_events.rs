//! The lifecycle events part of the FrogScript reference. A job in `scripts.toml` names these in `on`
//! and runs when one happens; the event's details are in the `STUDYFROG_EVENT` variable. Checked
//! against the event bus by `studyfrog-ui/tests/studio_reference.rs`.

use crate::wiki::{Category, Topic, topic_entry as t};

const E: Category = Category::LifecycleEvents;

/// One entry per event kind, in the order the bus lists them.
pub const EVENTS: &[Topic] = &[
    t!(
        "vault_opened",
        E,
        "on = [\"vault_opened\"]",
        "Nothing",
        "A vault was opened. Its folder is in the event's details.",
        "log_info(\"A vault was opened\")"
    ),
    t!(
        "vault_closed",
        E,
        "on = [\"vault_closed\"]",
        "Nothing",
        "A vault was closed, by the user or when the app quit.",
        "log_info(\"A vault was closed\")"
    ),
    t!(
        "item_created",
        E,
        "on = [\"item_created\"]",
        "Nothing",
        "A note, card or question was created in the open vault.",
        "log_info(\"An item was created\")"
    ),
    t!(
        "item_saved",
        E,
        "on = [\"item_saved\"]",
        "Nothing",
        "An item was saved after an edit.",
        "log_info(\"An item was saved\")"
    ),
    t!(
        "item_deleted",
        E,
        "on = [\"item_deleted\"]",
        "Nothing",
        "An item was deleted from the vault.",
        "log_info(\"An item was deleted\")"
    ),
    t!(
        "rehearsal_started",
        E,
        "on = [\"rehearsal_started\"]",
        "Nothing",
        "A rehearsal session began.",
        "log_info(\"A rehearsal started\")"
    ),
    t!(
        "rehearsal_stepped",
        E,
        "on = [\"rehearsal_stepped\"]",
        "Nothing",
        "One card or question of a rehearsal was answered.",
        "log_info(\"A rehearsal step was answered\")"
    ),
    t!(
        "rehearsal_finished",
        E,
        "on = [\"rehearsal_finished\"]",
        "Nothing",
        "A rehearsal session ended, with its results in the event's details.",
        "log_info(\"A rehearsal finished\")"
    ),
    t!(
        "exam_finished",
        E,
        "on = [\"exam_finished\"]",
        "Nothing",
        "An exam ended and its answers were marked.",
        "log_info(\"An exam finished\")"
    ),
    t!(
        "diary_entry_saved",
        E,
        "on = [\"diary_entry_saved\"]",
        "Nothing",
        "A diary entry was saved.",
        "log_info(\"A diary entry was saved\")"
    ),
    t!(
        "import_finished",
        E,
        "on = [\"import_finished\"]",
        "Nothing",
        "An import of files or a table into the vault finished.",
        "log_info(\"An import finished\")"
    ),
    t!(
        "backup_created",
        E,
        "on = [\"backup_created\"]",
        "Nothing",
        "A backup of the vault was written to disk.",
        "log_info(\"A backup was created\")"
    ),
    t!(
        "focus_block_finished",
        E,
        "on = [\"focus_block_finished\"]",
        "Nothing",
        "A timed focus block ended.",
        "log_info(\"A focus block finished\")"
    ),
    t!(
        "script_finished",
        E,
        "on = [\"script_finished\"]",
        "Nothing",
        "Another script run ended. Its job name and outcome are in the event's details.",
        "log_info(\"A script finished\")"
    ),
];
