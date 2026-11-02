//! The fields of a query: their names, the operators each takes and how a value for each is read.

use crate::query::{Field, Op};

/// Every field and the word that names it.
pub const FIELDS: [(Field, &str); 16] = [
    (Field::Text, "text"),
    (Field::Title, "title"),
    (Field::Type, "type"),
    (Field::Status, "status"),
    (Field::Stack, "stack"),
    (Field::Subject, "subject"),
    (Field::Teacher, "teacher"),
    (Field::Tag, "tag"),
    (Field::Priority, "priority"),
    (Field::Difficulty, "difficulty"),
    (Field::Familiarity, "familiarity"),
    (Field::Due, "due"),
    (Field::Reviews, "reviews"),
    (Field::Created, "created"),
    (Field::Updated, "updated"),
    (Field::Path, "path"),
];

/// The field a word names (without regard to case).
pub fn field_by_name(name: &str) -> Option<Field> {
    let lower = name.to_lowercase();
    FIELDS.iter().find(|(_, n)| *n == lower).map(|(f, _)| *f)
}

/// The word that names a field.
pub fn field_name(f: Field) -> &'static str {
    FIELDS.iter().find(|(x, _)| *x == f).map_or("", |(_, n)| n)
}

fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let keep = row[j + 1];
            row[j + 1] = (prev + usize::from(ca != cb)).min(row[j] + 1).min(keep + 1);
            prev = keep;
        }
    }
    row[b.len()]
}

/// The field whose name is within two edits of `name`, if any.
pub fn suggest(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    FIELDS
        .iter()
        .map(|(_, n)| (*n, distance(&lower, n)))
        .filter(|(_, d)| *d <= 2)
        .min_by_key(|(_, d)| *d)
        .map(|(n, _)| n)
}

/// The symbol of an operator.
pub fn op_symbol(op: Op) -> &'static str {
    match op {
        Op::Eq => "=",
        Op::Ne => "!=",
        Op::Lt => "<",
        Op::Le => "<=",
        Op::Gt => ">",
        Op::Ge => ">=",
        Op::Contains => "~",
        Op::NotContains => "!~",
    }
}

const EQUALITY: [Op; 2] = [Op::Eq, Op::Ne];
const TEXTUAL: [Op; 4] = [Op::Eq, Op::Ne, Op::Contains, Op::NotContains];
const ORDERED: [Op; 6] = [Op::Eq, Op::Ne, Op::Lt, Op::Le, Op::Gt, Op::Ge];

/// The operators a field takes (not counting `IN` and `IS EMPTY`).
pub fn ops_allowed(f: Field) -> &'static [Op] {
    match f {
        Field::Text => &[Op::Contains, Op::NotContains],
        Field::Title | Field::Path => &TEXTUAL,
        Field::Type
        | Field::Status
        | Field::Stack
        | Field::Subject
        | Field::Teacher
        | Field::Tag => &EQUALITY,
        _ => &ORDERED,
    }
}

/// Whether the field takes `IN`.
pub fn takes_in(f: Field) -> bool {
    matches!(
        f,
        Field::Type
            | Field::Status
            | Field::Stack
            | Field::Subject
            | Field::Teacher
            | Field::Tag
            | Field::Priority
            | Field::Difficulty
    )
}

/// Whether the field can be empty.
pub fn can_be_empty(f: Field) -> bool {
    matches!(
        f,
        Field::Subject
            | Field::Teacher
            | Field::Tag
            | Field::Difficulty
            | Field::Familiarity
            | Field::Due
            | Field::Created
            | Field::Updated
    )
}

/// Whether a result can be sorted by the field.
pub fn can_sort(f: Field) -> bool {
    !matches!(
        f,
        Field::Text | Field::Subject | Field::Teacher | Field::Tag
    )
}

/// The operators of a field in words: "`=`, `!=` or `IN`".
pub fn describe_ops(f: Field) -> String {
    let mut parts: Vec<String> = ops_allowed(f)
        .iter()
        .map(|o| format!("`{}`", op_symbol(*o)))
        .collect();
    if takes_in(f) {
        parts.push("`IN`".to_owned());
    }
    match parts.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
        _ => parts.join(""),
    }
}
