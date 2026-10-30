//! `subjects.toml` (vault root, optional): the teachers and the subjects of this vault. A subject has
//! one teacher; a teacher can have several subjects. Stacks name their subjects in their front matter.

use std::collections::HashSet;

use serde::Deserialize;

use crate::error::{ConfigError, config_error, from_toml};

/// File name in the vault root.
pub const SUBJECTS_FILE: &str = "subjects.toml";

/// A subject and the teacher who teaches it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject {
    /// Name, unique in the vault.
    pub name: String,
    /// The teacher's name, if the subject has one.
    pub teacher: Option<String>,
}

/// Every teacher and subject of a vault.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubjectBook {
    /// Teacher names, unique.
    pub teachers: Vec<String>,
    /// Subjects, unique by name.
    pub subjects: Vec<Subject>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    teacher: Vec<RawTeacher>,
    #[serde(default)]
    subject: Vec<RawSubject>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTeacher {
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSubject {
    name: String,
    teacher: Option<String>,
}

/// Why a book cannot be used, in words; `None` when it is fine.
pub fn book_problem(book: &SubjectBook) -> Option<String> {
    let blank = |s: &str| s.trim().is_empty() || s.contains('\n') || s.contains(',');
    let mut seen = HashSet::new();
    for t in &book.teachers {
        if blank(t) {
            return Some("a teacher's name must be one line without commas".to_owned());
        }
        if !seen.insert(t.to_lowercase()) {
            return Some(format!("the teacher \"{t}\" is listed twice"));
        }
    }
    let mut names = HashSet::new();
    for s in &book.subjects {
        if blank(&s.name) {
            return Some("a subject's name must be one line without commas".to_owned());
        }
        if !names.insert(s.name.to_lowercase()) {
            return Some(format!("the subject \"{}\" is listed twice", s.name));
        }
        let unknown = s.teacher.as_ref().filter(|t| !book.teachers.contains(t));
        if let Some(t) = unknown {
            return Some(format!(
                "the subject \"{}\" has an unknown teacher \"{t}\"",
                s.name
            ));
        }
    }
    None
}

/// Parse `subjects.toml`. Unknown keys, repeated names and unknown teachers are errors.
pub fn parse_subjects(text: &str) -> Result<SubjectBook, ConfigError> {
    let raw: RawFile = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    let book = SubjectBook {
        teachers: raw.teacher.into_iter().map(|t| t.name).collect(),
        subjects: raw
            .subject
            .into_iter()
            .map(|s| Subject {
                name: s.name,
                teacher: s.teacher,
            })
            .collect(),
    };
    match book_problem(&book) {
        Some(why) => Err(config_error(None, why)),
        None => Ok(book),
    }
}

/// A TOML basic string.
fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The file text for a book.
pub fn render_subjects(book: &SubjectBook) -> String {
    let mut out = String::from("# Teachers and subjects of this vault (edited in StudyFrog).\n");
    for t in &book.teachers {
        out.push_str(&format!("\n[[teacher]]\nname = {}\n", quoted(t)));
    }
    for s in &book.subjects {
        out.push_str(&format!("\n[[subject]]\nname = {}\n", quoted(&s.name)));
        if let Some(t) = &s.teacher {
            out.push_str(&format!("teacher = {}\n", quoted(t)));
        }
    }
    out
}
