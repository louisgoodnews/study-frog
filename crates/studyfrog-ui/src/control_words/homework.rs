//! The words of `homework ...`: `homework new "Essay" --due 2026-12-02 --time 09:00 --estimate 90
//! --priority high --stack french --checklist "Read;Draft"`, `homework advance <ULID>`, `homework tick
//! <ULID> 2`, `homework tab overdue`, `homework list`.

use studyfrog_calendar::homework::HomeworkPriority;
use studyfrog_calendar::homework_query::HomeworkTab;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::control_cmd::{HomeworkCmd, HomeworkNew};
use crate::control_words_items::{flag, need, split_flags};

fn item_id(text: &str) -> Result<Uid, String> {
    parse_ulid(text).ok_or_else(|| format!("{text} is not an item id (a ULID)"))
}

fn tab_named(word: &str) -> Result<HomeworkTab, String> {
    Ok(match word {
        "all" => HomeworkTab::All,
        "active" => HomeworkTab::Active,
        "due-today" => HomeworkTab::DueToday,
        "overdue" => HomeworkTab::Overdue,
        "completed" => HomeworkTab::Completed,
        other => {
            return Err(format!(
                "unknown tab {other} (all, active, due-today, overdue, completed)"
            ));
        }
    })
}

fn priority_named(word: &str) -> Result<HomeworkPriority, String> {
    Ok(match word {
        "low" => HomeworkPriority::Low,
        "medium" => HomeworkPriority::Medium,
        "high" => HomeworkPriority::High,
        other => return Err(format!("unknown priority {other} (low, medium, high)")),
    })
}

fn step(text: &str) -> Result<usize, String> {
    text.parse::<usize>()
        .ok()
        .filter(|n| *n >= 1)
        .ok_or_else(|| format!("{text} is not a step number (from 1)"))
}

/// The homework command the words after `homework` stand for.
pub fn parse(words: &[String]) -> Result<HomeworkCmd, String> {
    let (flags, rest) = split_flags(words, &[])?;
    Ok(match need(&rest, 0, "homework operation")? {
        "list" => HomeworkCmd::List,
        "tab" => HomeworkCmd::Tab(tab_named(need(&rest, 1, "tab")?)?),
        "new" => HomeworkCmd::New(HomeworkNew {
            title: need(&rest, 1, "title")?.to_owned(),
            due_date: flag(&flags, "due").map(str::to_owned),
            due_time: flag(&flags, "time").map(str::to_owned),
            estimate: flag(&flags, "estimate")
                .map(|v| {
                    v.parse::<u32>()
                        .map_err(|_| format!("--estimate needs minutes, not {v}"))
                })
                .transpose()?,
            priority: flag(&flags, "priority").map(priority_named).transpose()?,
            stack: flag(&flags, "stack").map(str::to_owned),
            checklist: flag(&flags, "checklist")
                .map(|c| {
                    c.split(';')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
        }),
        "advance" => HomeworkCmd::Advance(item_id(need(&rest, 1, "item ULID")?)?),
        "tick" => HomeworkCmd::Tick(
            item_id(need(&rest, 1, "item ULID")?)?,
            step(need(&rest, 2, "step number")?)?,
        ),
        other => return Err(format!("unknown homework operation {other}")),
    })
}
