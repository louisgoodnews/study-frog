//! Validation rules for issue files. Each rule returns error strings, empty when satisfied.

use std::collections::HashMap;

use crate::issues::Issue;

const STATUSES: [&str; 6] = [
    "backlog",
    "ready",
    "in_progress",
    "review",
    "done",
    "blocked",
];
const PHASES: [&str; 9] = ["m0", "m1", "m2", "m3", "m4", "m5", "m6", "m7", "m8"];
const ESTIMATES: [u32; 5] = [1, 2, 3, 5, 8];
const SECTIONS: [&str; 6] = [
    "## Context",
    "## Description",
    "## Acceptance Criteria",
    "## Design Notes",
    "## Dependencies",
    "## Open Questions",
];

type Index<'a> = HashMap<&'a str, &'a Issue>;

/// Run all rules over a set of issues.
pub fn check_all(issues: &[Issue]) -> Vec<String> {
    let mut index: Index = HashMap::new();
    let mut errors = Vec::new();
    for issue in issues {
        if index.insert(issue.front.id.as_str(), issue).is_some() {
            errors.push(err(issue, "duplicate id"));
        }
    }
    errors.extend(issues.iter().flat_map(|i| check_one(i, &index)));
    errors.extend(issues.iter().flat_map(|i| check_status(i, issues)));
    errors
}

fn check_one(issue: &Issue, index: &Index) -> Vec<String> {
    [
        check_identity(issue),
        check_fields(issue),
        check_links(issue, index),
        check_sections(issue),
    ]
    .concat()
}

fn err(issue: &Issue, msg: &str) -> String {
    format!("{}: {} {msg}", issue.path.display(), issue.front.id)
}

fn expected(kind: &str) -> Option<(&'static str, &'static str, usize)> {
    match kind {
        "epic" => Some(("EPIC", "epics", 3)),
        "story" => Some(("STORY", "stories", 4)),
        "task" => Some(("TASK", "tasks", 4)),
        "subtask" => Some(("SUB", "subtasks", 4)),
        "bug" => Some(("BUG", "bugs", 4)),
        _ => None,
    }
}

fn check_identity(issue: &Issue) -> Vec<String> {
    let f = &issue.front;
    let Some((prefix, dir, digits)) = expected(&f.kind) else {
        return vec![err(issue, &format!("unknown type `{}`", f.kind))];
    };
    let mut out = Vec::new();
    let valid_id =
        f.id.strip_prefix(prefix)
            .and_then(|r| r.strip_prefix('-'))
            .is_some_and(|n| n.len() == digits && n.bytes().all(|b| b.is_ascii_digit()));
    if !valid_id {
        out.push(err(
            issue,
            &format!("id must look like {prefix}-{}", "0".repeat(digits)),
        ));
    }
    if issue.dir != dir {
        out.push(err(
            issue,
            &format!("type `{}` belongs in `{dir}/`", f.kind),
        ));
    }
    if !file_name(issue).starts_with(&format!("{}-", f.id)) {
        out.push(err(issue, "file name must start with the id"));
    }
    out
}

fn file_name(issue: &Issue) -> String {
    issue
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_owned()
}

fn check_fields(issue: &Issue) -> Vec<String> {
    let f = &issue.front;
    let mut out = Vec::new();
    if !STATUSES.contains(&f.status.as_str()) {
        out.push(err(issue, &format!("invalid status `{}`", f.status)));
    }
    if f.title.trim().is_empty() {
        out.push(err(issue, "empty title"));
    }
    if f.acceptance_criteria.is_empty() {
        out.push(err(issue, "acceptance_criteria must not be empty"));
    }
    if !f.labels.iter().any(|l| PHASES.contains(&l.as_str())) {
        out.push(err(issue, "labels must include a phase label (m0..m8)"));
    }
    if f.estimate.is_some_and(|e| !ESTIMATES.contains(&e)) {
        out.push(err(issue, "estimate must be one of 1, 2, 3, 5, 8"));
    }
    out
}

fn parent_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "story" => Some("epic"),
        "task" => Some("story"),
        "subtask" => Some("task"),
        _ => None,
    }
}

fn check_links(issue: &Issue, index: &Index) -> Vec<String> {
    let f = &issue.front;
    let Some(want) = parent_kind(&f.kind) else {
        return Vec::new();
    };
    if f.parents.is_empty() {
        return vec![err(issue, &format!("needs a {want} parent"))];
    }
    f.parents
        .iter()
        .flat_map(|p| check_parent(issue, p, want, index))
        .collect()
}

fn check_parent(issue: &Issue, parent_id: &str, want: &str, index: &Index) -> Vec<String> {
    let Some(parent) = index.get(parent_id) else {
        return vec![err(issue, &format!("unknown parent {parent_id}"))];
    };
    let mut out = Vec::new();
    if parent.front.kind != want {
        out.push(err(issue, &format!("parent {parent_id} must be a {want}")));
    }
    if issue.front.epic != epic_of(parent) {
        out.push(err(
            issue,
            &format!("epic differs from parent {parent_id}'s epic"),
        ));
    }
    out
}

fn epic_of(issue: &Issue) -> Option<String> {
    if issue.front.kind == "epic" {
        Some(issue.front.id.clone())
    } else {
        issue.front.epic.clone()
    }
}

fn check_sections(issue: &Issue) -> Vec<String> {
    let mut cursor = 0;
    let lines: Vec<&str> = issue.body.lines().collect();
    SECTIONS
        .iter()
        .filter_map(|s| match lines.iter().skip(cursor).position(|l| l == s) {
            Some(offset) => {
                cursor += offset + 1;
                None
            }
            None => Some(err(issue, &format!("missing or misordered section `{s}`"))),
        })
        .collect()
}

/// Status must agree with the hierarchy: a parent is done exactly when all its children are, and a
/// done item has no unchecked acceptance criteria.
fn check_status(issue: &Issue, all: &[Issue]) -> Vec<String> {
    let mut out = Vec::new();
    let done = issue.front.status == "done";
    if done
        && issue
            .body
            .lines()
            .any(|l| l.trim_start().starts_with("- [ ]"))
    {
        out.push(err(issue, "is done but has unchecked acceptance criteria"));
    }
    let children: Vec<&Issue> = all
        .iter()
        .filter(|c| c.front.parents.iter().any(|p| *p == issue.front.id))
        .collect();
    let all_done = !children.is_empty() && children.iter().all(|c| c.front.status == "done");
    if matches!(issue.front.kind.as_str(), "epic" | "story") && !children.is_empty() {
        if all_done && !done {
            out.push(err(issue, "all children are done but this item is not"));
        }
        if done && !all_done {
            out.push(err(issue, "is done but has children that are not"));
        }
    }
    out
}
