//! Working the vault like a wiki: what the model is told, and the changes it may propose. The model
//! proposes; nothing is written until the user applies a proposal (ADR 0009).

use serde_json::Value;

/// One item of the vault as the model sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextItem {
    /// The item's id as text.
    pub uid: String,
    /// `note`, `card` or `question`.
    pub kind: String,
    /// Title.
    pub title: String,
    /// The stack folder it is in.
    pub stack: String,
    /// Its text: the body, or both sides.
    pub text: String,
}

/// A change the model proposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// A new note.
    CreateNote {
        /// The stack folder.
        stack: String,
        /// Title.
        title: String,
        /// Markdown body.
        body: String,
    },
    /// A new card.
    CreateCard {
        /// The stack folder.
        stack: String,
        /// Front.
        front: String,
        /// Back.
        back: String,
    },
    /// A new question.
    CreateQuestion {
        /// The stack folder.
        stack: String,
        /// The question.
        prompt: String,
        /// The reference answer.
        reference: String,
    },
    /// New text for parts of an item (`body`, `front`, `back`, `prompt` or `reference`).
    UpdateItem {
        /// The item's id as text.
        uid: String,
        /// The parts and their new text.
        fields: Vec<(String, String)>,
    },
}

/// What the model proposed, checked.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Proposal {
    /// What the model says it did and why.
    pub summary: String,
    /// The changes that passed the checks.
    pub changes: Vec<Change>,
    /// What was left out, each in a sentence.
    pub rejected: Vec<String>,
}

/// Most changes one proposal may hold.
pub const MAX_CHANGES: usize = 40;
/// Longest text of one part.
pub const MAX_TEXT: usize = 20_000;
const FIELDS: [&str; 5] = ["body", "front", "back", "prompt", "reference"];

/// The instructions for working the wiki.
pub fn wiki_system() -> String {
    "You maintain a personal knowledge vault of Markdown notes, flashcards and questions, the way a careful \
     wiki editor would: keep each note on one topic, link related notes with [[Title]] wikilinks, keep an \
     'Index' note in each stack that lists its notes as links, and turn facts worth remembering into flashcards. \
     You only propose changes; the user decides what is applied. Never invent facts that are not in the \
     sources you were given. Answer with ONE JSON object and nothing else, in this shape: \
     {\"summary\": \"what you did and why\", \"changes\": [\
     {\"action\": \"create_note\", \"stack\": \"<folder>\", \"title\": \"...\", \"body\": \"markdown\"}, \
     {\"action\": \"create_card\", \"stack\": \"<folder>\", \"front\": \"...\", \"back\": \"...\"}, \
     {\"action\": \"create_question\", \"stack\": \"<folder>\", \"prompt\": \"...\", \"reference\": \"...\"}, \
     {\"action\": \"update_item\", \"uid\": \"<id>\", \"fields\": {\"body\": \"new markdown\"}}]}. \
     Use only the stack folders and ids that appear in the vault listing."
        .to_owned()
}

/// The user turn: what to do, the stacks, and the items.
pub fn wiki_user(instruction: &str, stacks: &[(String, String)], items: &[ContextItem]) -> String {
    let mut out = format!("Task: {}\n\nStacks (folder: title):\n", instruction.trim());
    for (folder, title) in stacks {
        out.push_str(&format!("- {folder}: {title}\n"));
    }
    out.push_str("\nItems:\n");
    for i in items {
        out.push_str(&format!(
            "\n### id={} kind={} stack={} title={}\n{}\n",
            i.uid, i.kind, i.stack, i.title, i.text
        ));
    }
    out
}

/// The JSON object inside a reply (models sometimes wrap it in a fence or a sentence).
pub fn json_in(reply: &str) -> Option<Value> {
    let (start, end) = (reply.find('{')?, reply.rfind('}')?);
    (start < end)
        .then(|| serde_json::from_str(&reply[start..=end]).ok())
        .flatten()
}

fn text_of(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.chars().count() <= MAX_TEXT)
        .map(str::to_owned)
}

fn line_of(v: &Value, key: &str) -> Option<String> {
    text_of(v, key).filter(|s| s.chars().count() <= 200 && !s.chars().any(char::is_control))
}

fn change_of(v: &Value, uids: &[String], stacks: &[String]) -> Result<Change, String> {
    let action = v.get("action").and_then(Value::as_str).unwrap_or("");
    let stack = || {
        text_of(v, "stack")
            .filter(|s| stacks.contains(s))
            .ok_or_else(|| format!("{action}: the stack is not one of the listed folders"))
    };
    let need = |k: &str, t: Option<String>| {
        t.ok_or_else(|| format!("{action}: \"{k}\" is missing or not usable"))
    };
    match action {
        "create_note" => Ok(Change::CreateNote {
            stack: stack()?,
            title: need("title", line_of(v, "title"))?,
            body: need("body", text_of(v, "body"))?,
        }),
        "create_card" => Ok(Change::CreateCard {
            stack: stack()?,
            front: need("front", text_of(v, "front"))?,
            back: need("back", text_of(v, "back"))?,
        }),
        "create_question" => Ok(Change::CreateQuestion {
            stack: stack()?,
            prompt: need("prompt", text_of(v, "prompt"))?,
            reference: need("reference", text_of(v, "reference"))?,
        }),
        "update_item" => {
            let uid = need("uid", text_of(v, "uid"))?;
            if !uids.contains(&uid) {
                return Err(format!(
                    "update_item: {uid} is not one of the items you were shown"
                ));
            }
            let fields: Vec<(String, String)> = v
                .get("fields")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
                .filter(|(k, _)| FIELDS.contains(&k.as_str()))
                .filter_map(|(k, t)| t.as_str().map(|t| (k.clone(), t.trim().to_owned())))
                .filter(|(_, t)| !t.is_empty() && t.chars().count() <= MAX_TEXT)
                .collect();
            if fields.is_empty() {
                return Err("update_item: no usable text to change".to_owned());
            }
            Ok(Change::UpdateItem { uid, fields })
        }
        other => Err(format!("an unknown action \"{other}\"")),
    }
}

/// Read and check a reply. `uids` are the items and `stacks` the folders the model was shown.
pub fn parse_proposal(reply: &str, uids: &[String], stacks: &[String]) -> Result<Proposal, String> {
    let v = json_in(reply)
        .ok_or("The model did not answer with the JSON proposal that was asked for.")?;
    let summary = v
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_owned();
    let list = v
        .get("changes")
        .and_then(Value::as_array)
        .ok_or("The proposal has no \"changes\" list.")?;
    let mut out = Proposal {
        summary,
        ..Proposal::default()
    };
    for (i, c) in list.iter().enumerate() {
        if out.changes.len() >= MAX_CHANGES {
            out.rejected.push(format!(
                "Only the first {MAX_CHANGES} changes are kept; {} were left out.",
                list.len() - i
            ));
            break;
        }
        match change_of(c, uids, stacks) {
            Ok(change) => out.changes.push(change),
            Err(why) => out.rejected.push(format!("Change {}: {why}", i + 1)),
        }
    }
    Ok(out)
}
