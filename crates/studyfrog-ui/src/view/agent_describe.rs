//! How a proposed change is shown for review: a headline and the text it would write.

use studyfrog_agent::wiki::{Change, DiaryChange};

/// The headline and the text of a change.
pub fn describe(c: &Change) -> (String, String) {
    match c {
        Change::CreateNote { stack, title, body } => {
            (format!("New note \"{title}\" in {stack}"), body.clone())
        }
        Change::CreateCard {
            stack, front, back, ..
        } => (
            format!("New card in {stack}: {front}"),
            format!("Front: {front}\nBack: {back}"),
        ),
        Change::CreateQuestion {
            stack,
            prompt,
            reference,
            ..
        } => (
            format!("New question in {stack}: {prompt}"),
            format!("Prompt: {prompt}\nReference answer: {reference}"),
        ),
        Change::CreatePage { name, text } => (format!("New help page \"{name}\""), text.clone()),
        Change::UpdatePage { name, text } => {
            (format!("Change your help page \"{name}\""), text.clone())
        }
        Change::UpdateItem { uid, fields } => (
            format!("Change item {uid}"),
            fields
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        Change::LogDiary(d) => (format!("New diary entry for {}", d.date), diary_text(d)),
        Change::PlanSession(p) => (
            format!(
                "Plan \"{}\" on {} at {:02}:{:02} ({} min)",
                p.title,
                p.date,
                p.start_minute / 60,
                p.start_minute % 60,
                p.duration_min
            ),
            format!(
                "Kind: {}\nStack: {}\n{}",
                p.kind,
                p.stack.as_deref().unwrap_or("none"),
                p.notes
            ),
        ),
    }
}

fn diary_text(d: &DiaryChange) -> String {
    let ratings: Vec<String> = d.ratings.iter().map(|(k, n)| format!("{k}: {n}")).collect();
    let minutes = d
        .minutes
        .map_or_else(String::new, |m| format!("Studied {m} min. "));
    format!("{minutes}Ratings: {}\n{}", ratings.join(", "), d.body)
}
