//! Renders `BOARD.md` (a Kanban view) from issue front-matter, so it cannot drift from the issues.

use std::fmt::Write;

use crate::issues::Issue;

/// Columns, left to right. Must match the status values in `rules`.
const COLUMNS: [(&str, &str); 6] = [
    ("backlog", "Backlog"),
    ("ready", "Ready"),
    ("in_progress", "In progress"),
    ("review", "Review"),
    ("blocked", "Blocked"),
    ("done", "Done"),
];

/// Maximum simultaneous `in_progress` work items.
pub const WIP_LIMIT: usize = 3;

const HEADER: &str = "# Board\n\n\
> Generated from `docs/issues/` by `scripts/board.sh`. **Do not edit by hand.** Change an issue's\n\
> `status:` and regenerate; `scripts/check.sh` fails if this file is stale.\n\
> Workflow and WIP rules: [issues/README](docs/issues/README.md).\n";

/// Work items shown as cards: everything below story level, plus bugs.
fn is_card(issue: &Issue) -> bool {
    matches!(issue.front.kind.as_str(), "task" | "subtask" | "bug")
}

fn in_column<'a>(issues: &'a [Issue], status: &str, pick: fn(&Issue) -> bool) -> Vec<&'a Issue> {
    let mut found: Vec<&Issue> = issues
        .iter()
        .filter(|i| pick(i) && i.front.status == status)
        .collect();
    found.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    found
}

fn cell(issue: &Issue) -> String {
    let f = &issue.front;
    let points = f.estimate.map(|e| format!(" ({e})")).unwrap_or_default();
    let title = f.title.replace('|', "\\|");
    format!("[{}]({}) {title}{points}", f.id, issue.path.display())
}

fn heading(status: &str, label: &str, count: usize) -> String {
    match status {
        "in_progress" => format!("{label} ({count}/{WIP_LIMIT})"),
        _ => format!("{label} ({count})"),
    }
}

fn card_table(issues: &[Issue]) -> String {
    let cols: Vec<Vec<&Issue>> = COLUMNS
        .iter()
        .map(|(status, _)| in_column(issues, status, is_card))
        .collect();
    let rows = cols.iter().map(Vec::len).max().unwrap_or(0);
    let titles: Vec<String> = COLUMNS
        .iter()
        .zip(&cols)
        .map(|((status, label), col)| heading(status, label, col.len()))
        .collect();
    let mut out = format!(
        "| {} |\n|{}\n",
        titles.join(" | "),
        " --- |".repeat(COLUMNS.len())
    );
    for row in 0..rows {
        let cells: Vec<String> = cols
            .iter()
            .map(|c| c.get(row).map(|i| cell(i)).unwrap_or_default())
            .collect();
        let _ = writeln!(out, "| {} |", cells.join(" | "));
    }
    out
}

fn children<'a>(issues: &'a [Issue], parent: &str) -> Vec<&'a Issue> {
    issues
        .iter()
        .filter(|i| i.front.parents.iter().any(|p| p == parent))
        .collect()
}

fn progress(issues: &[Issue], parent: &Issue) -> String {
    let kids = children(issues, &parent.front.id);
    let done = kids.iter().filter(|k| k.front.status == "done").count();
    format!("{done}/{}", kids.len())
}

fn rollup(issues: &[Issue], kind: &str, title: &str) -> String {
    let mut parents: Vec<&Issue> = issues.iter().filter(|i| i.front.kind == kind).collect();
    parents.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    let mut out = format!("\n## {title}\n\n| Item | Status | Done |\n| --- | --- | --- |\n");
    for p in parents {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            cell(p),
            p.front.status,
            progress(issues, p)
        );
    }
    out
}

/// The full `BOARD.md` text.
pub fn render(issues: &[Issue]) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    out.push_str(&card_table(issues));
    out.push_str(&rollup(issues, "story", "Stories"));
    out.push_str(&rollup(issues, "epic", "Epics"));
    out
}

/// Errors for WIP-limit violations.
pub fn wip_errors(issues: &[Issue]) -> Vec<String> {
    let active = in_column(issues, "in_progress", is_card).len();
    if active > WIP_LIMIT {
        vec![format!(
            "board: {active} items in_progress exceeds WIP limit {WIP_LIMIT}"
        )]
    } else {
        Vec::new()
    }
}
