//! The words of `item ...` and `stack ...`, and the flag reader the other commands share.

use studyfrog_service::create_more::NewKind;

use crate::control_cmd::{EditTarget, ItemCmd, NewItemArgs, StackCmd};
use crate::view::choice_body::{AnswerType, answer_type_named};

/// Flags with their values (`""` for a bare flag), in order.
pub type Flags<'a> = Vec<(&'a str, &'a str)>;

/// The value of `--flag` among `words`, and the words with every flag and its value taken out.
/// Flags in `bare` take no value.
pub fn split_flags<'a>(
    words: &'a [String],
    bare: &[&str],
) -> Result<(Flags<'a>, Vec<&'a str>), String> {
    let (mut flags, mut rest, mut it) = (Vec::new(), Vec::new(), words.iter());
    while let Some(w) = it.next() {
        match w.strip_prefix("--") {
            Some(name) if bare.contains(&name) => flags.push((name, "")),
            Some(name) => {
                let v = it.next().ok_or_else(|| format!("--{name} needs a value"))?;
                flags.push((name, v.as_str()));
            }
            None => rest.push(w.as_str()),
        }
    }
    Ok((flags, rest))
}

/// The last value given for `--name`.
pub fn flag<'a>(flags: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    flags
        .iter()
        .rev()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| *v)
}

/// The positional word at `at`, or an error naming what is missing.
pub fn need<'a>(rest: &[&'a str], at: usize, what: &str) -> Result<&'a str, String> {
    rest.get(at)
        .copied()
        .ok_or_else(|| format!("missing {what}"))
}

fn kind_named(word: &str) -> Result<NewKind, String> {
    Ok(match word {
        "stack" => NewKind::Stack,
        "note" => NewKind::Note,
        "card" | "flashcard" => NewKind::Card,
        "question" => NewKind::Question,
        other => {
            return Err(format!(
                "unknown kind {other} (stack, note, card, question)"
            ));
        }
    })
}

fn target_named(word: &str) -> Result<EditTarget, String> {
    Ok(match word {
        "title" => EditTarget::Title,
        "tags" => EditTarget::Tags,
        "body" | "front" | "prompt" | "description" | "text" => EditTarget::Field(0),
        "back" | "reference" | "answer" => EditTarget::Field(1),
        other => match other.strip_prefix("field").map(str::parse) {
            Some(Ok(n)) => EditTarget::Field(n),
            _ => return Err(format!("unknown field {other}")),
        },
    })
}

/// Option numbers (1-based) from words such as `1,3` or `1 3`.
pub fn numbers(words: &[String]) -> Result<Vec<usize>, String> {
    let list = words
        .iter()
        .flat_map(|w| w.split(','))
        .filter(|s| !s.is_empty())
        .map(|s| match s.parse::<usize>() {
            Ok(n) if n >= 1 => Ok(n),
            _ => Err(format!("option numbers start at 1, not {s:?}")),
        })
        .collect::<Result<Vec<usize>, String>>()?;
    if list.is_empty() {
        return Err("missing option number (rehearse choose 1,3)".to_owned());
    }
    Ok(list)
}

/// The fields of `item create <kind> <title>`; `--answer-type` applies to questions only.
fn new_item(rest: &[&str], flags: &Flags<'_>) -> Result<NewItemArgs, String> {
    let kind = kind_named(need(rest, 1, "kind")?)?;
    let answer_type = match flag(flags, "answer-type") {
        None => AnswerType::Open,
        Some(word) => answer_type_named(word).ok_or_else(|| {
            format!("unknown answer type {word} (open, single, multiple)")
        })?,
    };
    if answer_type != AnswerType::Open && kind != NewKind::Question {
        return Err("--answer-type applies to questions only".to_owned());
    }
    Ok(NewItemArgs {
        kind,
        title: need(rest, 2, "title")?.to_owned(),
        text: flag(flags, "text").unwrap_or_default().to_owned(),
        answer: flag(flags, "answer").unwrap_or_default().to_owned(),
        tags: flag(flags, "tags").unwrap_or_default().to_owned(),
        stack: flag(flags, "stack").map(str::to_owned),
        answer_type,
    })
}

/// The words after `item`.
pub fn item(words: &[String]) -> Result<ItemCmd, String> {
    let (flags, rest) = split_flags(words, &[])?;
    let item = || need(&rest, 1, "item (ULID or key)").map(str::to_owned);
    Ok(match need(&rest, 0, "item operation")? {
        "create" => ItemCmd::Create(new_item(&rest, &flags)?),
        "open" => ItemCmd::Open(item()?),
        "delete" => ItemCmd::Delete(item()?),
        "edit" => ItemCmd::Edit(
            item()?,
            target_named(need(&rest, 2, "field")?)?,
            rest.get(3..).unwrap_or_default().join(" "),
        ),
        "color" | "colour" => {
            let color = need(&rest, 2, "colour (a word, #RRGGBB or none)")?;
            ItemCmd::Color(item()?, (color != "none").then(|| color.to_owned()))
        }
        "move" => ItemCmd::Move(item()?, need(&rest, 2, "stack folder")?.to_owned()),
        other => return Err(format!("unknown item operation {other}")),
    })
}

/// The words after `stack`.
pub fn stack(words: &[String]) -> Result<StackCmd, String> {
    let (flags, rest) = split_flags(words, &[])?;
    let folder = || need(&rest, 1, "stack folder").map(str::to_owned);
    Ok(match need(&rest, 0, "stack operation")? {
        "create" => StackCmd::Create(folder()?, flag(&flags, "parent").map(str::to_owned)),
        "open" => StackCmd::Open(folder()?),
        "settings" => StackCmd::Settings(folder()?),
        "move" => StackCmd::Move(folder()?, rest.get(2).copied().unwrap_or("").to_owned()),
        "delete" => StackCmd::Delete(folder()?),
        other => return Err(format!("unknown stack operation {other}")),
    })
}
