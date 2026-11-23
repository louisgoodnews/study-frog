//! Pure Markdown rendering of a classroom session: header, roster, transcript, assignments,
//! submissions and grades. No I/O; the caller decides where the text goes.

use studyfrog_core::civil::civil_from_days;
use studyfrog_core::classroom::{
    Assignment, AssignmentKind, ClassroomSession, GradeReport, Owner, Participant, Submission,
};
use studyfrog_core::classroom_ops::participant_name;
use studyfrog_core::time::UnixMs;

const DAY_MS: i64 = 86_400_000;
const MINUTE_MS: i64 = 60_000;

/// The whole session as a Markdown document, ending with a newline.
pub fn render_session_markdown(s: &ClassroomSession) -> String {
    let mut lines = vec![format!("# {}", s.title), String::new()];
    lines.push(format!("Created {}", stamp(s.created_at)));
    lines.push(String::new());
    push_roster(&mut lines, s);
    push_transcript(&mut lines, s);
    push_assignments(&mut lines, s);
    push_submissions(&mut lines, s);
    let mut out = lines.join("\n");
    while out.ends_with("\n\n") {
        out.pop();
    }
    out.push('\n');
    out
}

/// A UTC date and minute, e.g. `2026-11-23 09:05 UTC`.
fn stamp(at: UnixMs) -> String {
    let day = at.0.div_euclid(DAY_MS);
    let minute = at.0.rem_euclid(DAY_MS) / MINUTE_MS;
    let (y, m, d) = civil_from_days(day);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02} UTC",
        minute / 60,
        minute % 60
    )
}

/// The display name of a participant index, or a placeholder for an index that does not exist.
fn speaker_name(s: &ClassroomSession, index: usize) -> &str {
    s.participants
        .get(index)
        .map_or("unknown", participant_name)
}

fn role_of(p: &Participant) -> &'static str {
    match p {
        Participant::Teacher { .. } => "Teacher",
        Participant::SimulatedStudent { .. } => "Simulated student",
        Participant::User { .. } => "You",
    }
}

fn push_roster(lines: &mut Vec<String>, s: &ClassroomSession) {
    lines.push("## Participants".to_owned());
    lines.push(String::new());
    for p in &s.participants {
        lines.push(format!("- {}: {}", role_of(p), participant_name(p)));
    }
    lines.push(String::new());
}

fn push_transcript(lines: &mut Vec<String>, s: &ClassroomSession) {
    lines.push("## Transcript".to_owned());
    lines.push(String::new());
    for t in &s.turns {
        lines.push(format!(
            "**{}** ({}): {}",
            speaker_name(s, t.speaker),
            stamp(t.at),
            t.text
        ));
        lines.push(String::new());
    }
}

fn push_assignments(lines: &mut Vec<String>, s: &ClassroomSession) {
    lines.push("## Assignments".to_owned());
    lines.push(String::new());
    for a in &s.assignments {
        lines.push(format!("### {}", a.title));
        lines.push(String::new());
        lines.push(a.prompt.clone());
        lines.push(String::new());
        lines.push(who_line(s, a));
        lines.push(String::new());
    }
}

/// Who the assignment is for: the assignee, or each group with its members.
fn who_line(s: &ClassroomSession, a: &Assignment) -> String {
    match &a.kind {
        AssignmentKind::Solo { assignee } => format!("For: {}", speaker_name(s, *assignee)),
        AssignmentKind::Group { groups } => {
            let parts: Vec<String> = groups
                .iter()
                .map(|g| {
                    let members: Vec<&str> =
                        g.members.iter().map(|&m| speaker_name(s, m)).collect();
                    format!("{} ({})", g.name, members.join(", "))
                })
                .collect();
            format!("Groups: {}", parts.join("; "))
        }
    }
}

fn push_submissions(lines: &mut Vec<String>, s: &ClassroomSession) {
    lines.push("## Submissions and grades".to_owned());
    lines.push(String::new());
    for sub in &s.submissions {
        lines.push(submission_heading(s, sub));
        lines.push(String::new());
        lines.push(format!("Turned in {}", stamp(sub.submitted_at)));
        lines.push(String::new());
        lines.push(sub.text.clone());
        lines.push(String::new());
        lines.push(grade_line(sub.grade.as_ref()));
        lines.push(String::new());
    }
}

fn submission_heading(s: &ClassroomSession, sub: &Submission) -> String {
    let title = s
        .assignments
        .iter()
        .find(|a| a.id == sub.assignment)
        .map_or("unknown assignment", |a| a.title.as_str());
    format!("### {title}: {}", owner_label(s, sub))
}

/// The participant's name, or the group's name within its assignment.
fn owner_label(s: &ClassroomSession, sub: &Submission) -> String {
    match sub.owner {
        Owner::Participant(p) => speaker_name(s, p).to_owned(),
        Owner::Group(g) => s
            .assignments
            .iter()
            .find(|a| a.id == sub.assignment)
            .and_then(|a| match &a.kind {
                AssignmentKind::Group { groups } => groups.get(g),
                AssignmentKind::Solo { .. } => None,
            })
            .map_or_else(|| "unknown group".to_owned(), |g| g.name.clone()),
    }
}

fn grade_line(grade: Option<&GradeReport>) -> String {
    match grade {
        None => "Not graded yet.".to_owned(),
        Some(g) if g.feedback.trim().is_empty() => format!("Grade: {}/{}", g.points, g.max),
        Some(g) => format!("Grade: {}/{}. {}", g.points, g.max, g.feedback),
    }
}
