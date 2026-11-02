//! Running a query over the items of a vault: which items match and in what order. Pure and in memory: the
//! query is a tree of typed values, so nothing a user types is ever put into SQL.

use std::cmp::Ordering;

use crate::civil::{add_months, date_from_days, date_to_days};
use crate::event::Difficulty;
use crate::plan::ItemKind;
use crate::priority::{PRIORITIES, Priority};
use crate::query::{Cond, DateUnit, DateVal, Field, Op, Order, Query, Val};
use crate::status::Status;

/// One item as a query sees it. Text is compared without regard to case.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The file's path in the vault.
    pub path: String,
    /// The title.
    pub title: String,
    /// The text of the item (front and back, body).
    pub body: String,
    /// Note, card or question.
    pub kind: ItemKind,
    /// The effective status.
    pub status: Status,
    /// The folder and the title of the item's stack and of every stack above it.
    pub stacks: Vec<String>,
    /// The subjects of the stack, inherited.
    pub subjects: Vec<String>,
    /// The teachers of those subjects.
    pub teachers: Vec<String>,
    /// The tags.
    pub tags: Vec<String>,
    /// The effective priority.
    pub priority: Priority,
    /// The effective difficulty, if any.
    pub difficulty: Option<Difficulty>,
    /// Familiarity from 0 to 1; none for an item with no history.
    pub familiarity: Option<f32>,
    /// The day of the next review, counted from 1970-01-01.
    pub due_day: Option<i64>,
    /// How many reviews it has.
    pub reviews: u32,
    /// The day it was created.
    pub created_day: Option<i64>,
    /// The day it was last updated.
    pub updated_day: Option<i64>,
}

fn day_of(d: DateVal, today: i64) -> i64 {
    let shifted =
        |months: i64| add_months(date_from_days(today), months).map_or(today, date_to_days);
    match d {
        DateVal::Day(day) => day,
        DateVal::Relative(n, DateUnit::Day) => today + n,
        DateVal::Relative(n, DateUnit::Week) => today + 7 * n,
        DateVal::Relative(n, DateUnit::Month) => shifted(n),
        DateVal::Relative(n, DateUnit::Year) => shifted(12 * n),
    }
}

/// A familiarity as a number to compare with what the user typed: four decimals, so that 0.9 stored as `f32` is 0.9.
fn rounded(f: f32) -> f64 {
    (f64::from(f) * 10_000.0).round() / 10_000.0
}

fn priority_rank(p: Priority) -> i64 {
    PRIORITIES.iter().position(|x| *x == p).unwrap_or(0) as i64
}

fn difficulty_rank(d: Difficulty) -> i64 {
    match d {
        Difficulty::Easy => 0,
        Difficulty::Medium => 1,
        Difficulty::Difficult => 2,
    }
}

fn compare<T: PartialOrd>(op: Op, have: T, want: T) -> bool {
    match op {
        Op::Eq => have == want,
        Op::Ne => have != want,
        Op::Lt => have < want,
        Op::Le => have <= want,
        Op::Gt => have > want,
        Op::Ge => have >= want,
        Op::Contains | Op::NotContains => false,
    }
}

/// A comparison against a value that may be missing: a missing value matches only `!=`.
fn compare_opt<T: PartialOrd>(op: Op, have: Option<T>, want: T) -> bool {
    match have {
        Some(h) => compare(op, h, want),
        None => op == Op::Ne,
    }
}

fn text_compare(op: Op, have: &str, want: &str) -> bool {
    let (h, w) = (have.to_lowercase(), want.to_lowercase());
    match op {
        Op::Eq => h == w,
        Op::Ne => h != w,
        Op::Contains => h.contains(&w),
        Op::NotContains => !h.contains(&w),
        _ => false,
    }
}

fn any_equal(have: &[String], want: &str) -> bool {
    have.iter()
        .any(|h| h.eq_ignore_ascii_case(want) || h.to_lowercase() == want.to_lowercase())
}

fn list_compare(op: Op, have: &[String], want: &str) -> bool {
    let found = any_equal(have, want);
    if op == Op::Ne { !found } else { found }
}

fn cmp(row: &Row, today: i64, (field, op, val): (Field, Op, &Val)) -> bool {
    match (field, val) {
        (Field::Text, Val::Text(w)) => text_compare(op, &format!("{} {}", row.title, row.body), w),
        (Field::Title, Val::Text(w)) => text_compare(op, &row.title, w),
        (Field::Path, Val::Text(w)) => text_compare(op, &row.path, w),
        (Field::Type, Val::Kind(k)) => compare(op, row.kind == *k, true),
        (Field::Status, Val::Status(s)) => compare(op, row.status == *s, true),
        (Field::Stack, Val::Text(w)) => list_compare(op, &row.stacks, w),
        (Field::Subject, Val::Text(w)) => list_compare(op, &row.subjects, w),
        (Field::Teacher, Val::Text(w)) => list_compare(op, &row.teachers, w),
        (Field::Tag, Val::Text(w)) => list_compare(op, &row.tags, w),
        (Field::Priority, Val::Priority(p)) => {
            compare(op, priority_rank(row.priority), priority_rank(*p))
        }
        (Field::Difficulty, Val::Difficulty(d)) => {
            compare_opt(op, row.difficulty.map(difficulty_rank), difficulty_rank(*d))
        }
        (Field::Familiarity, Val::Num(n)) => compare_opt(op, row.familiarity.map(rounded), *n),
        (Field::Reviews, Val::Int(n)) => compare(op, i64::from(row.reviews), *n),
        (Field::Due, Val::Date(d)) => compare_opt(op, row.due_day, day_of(*d, today)),
        (Field::Created, Val::Date(d)) => compare_opt(op, row.created_day, day_of(*d, today)),
        (Field::Updated, Val::Date(d)) => compare_opt(op, row.updated_day, day_of(*d, today)),
        _ => false,
    }
}

fn is_empty(row: &Row, field: Field) -> bool {
    match field {
        Field::Subject => row.subjects.is_empty(),
        Field::Teacher => row.teachers.is_empty(),
        Field::Tag => row.tags.is_empty(),
        Field::Difficulty => row.difficulty.is_none(),
        Field::Familiarity => row.familiarity.is_none(),
        Field::Due => row.due_day.is_none(),
        Field::Created => row.created_day.is_none(),
        Field::Updated => row.updated_day.is_none(),
        _ => false,
    }
}

fn holds(c: &Cond, row: &Row, today: i64) -> bool {
    match c {
        Cond::Cmp(f, op, v) => cmp(row, today, (*f, *op, v)),
        Cond::In(f, vals, negated) => {
            vals.iter().any(|v| cmp(row, today, (*f, Op::Eq, v))) != *negated
        }
        Cond::Empty(f, negated) => is_empty(row, *f) != *negated,
        Cond::And(parts) => parts.iter().all(|p| holds(p, row, today)),
        Cond::Or(parts) => parts.iter().any(|p| holds(p, row, today)),
        Cond::Not(inner) => !holds(inner, row, today),
    }
}

/// Whether the item satisfies the query's condition. `today` is a day counted from 1970-01-01.
pub fn matches(q: &Query, row: &Row, today: i64) -> bool {
    q.cond.as_ref().is_none_or(|c| holds(c, row, today))
}

fn missing_last<T: PartialOrd>(a: Option<T>, b: Option<T>, descending: bool) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => {
            let o = x.partial_cmp(&y).unwrap_or(Ordering::Equal);
            if descending { o.reverse() } else { o }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn by_field(a: &Row, b: &Row, o: Order) -> Ordering {
    let d = o.descending;
    let text = |x: &str, y: &str| missing_last(Some(x.to_lowercase()), Some(y.to_lowercase()), d);
    match o.field {
        Field::Title => text(&a.title, &b.title),
        Field::Path | Field::Stack => text(&a.path, &b.path),
        Field::Type => missing_last(Some(a.kind as u8), Some(b.kind as u8), d),
        Field::Status => missing_last(Some(a.status as u8), Some(b.status as u8), d),
        Field::Priority => missing_last(
            Some(priority_rank(a.priority)),
            Some(priority_rank(b.priority)),
            d,
        ),
        Field::Difficulty => missing_last(
            a.difficulty.map(difficulty_rank),
            b.difficulty.map(difficulty_rank),
            d,
        ),
        Field::Familiarity => missing_last(a.familiarity, b.familiarity, d),
        Field::Due => missing_last(a.due_day, b.due_day, d),
        Field::Reviews => missing_last(Some(a.reviews), Some(b.reviews), d),
        Field::Created => missing_last(a.created_day, b.created_day, d),
        Field::Updated => missing_last(a.updated_day, b.updated_day, d),
        Field::Text | Field::Subject | Field::Teacher | Field::Tag => Ordering::Equal,
    }
}

/// The indices of the rows that match, in the order the query asks for (items equal on every key are by path).
pub fn run(q: &Query, rows: &[Row], today: i64) -> Vec<usize> {
    let mut found: Vec<usize> = (0..rows.len())
        .filter(|i| matches(q, &rows[*i], today))
        .collect();
    found.sort_by(|a, b| {
        let keys = q.order.iter().map(|o| by_field(&rows[*a], &rows[*b], *o));
        keys.fold(Ordering::Equal, Ordering::then)
            .then_with(|| rows[*a].path.cmp(&rows[*b].path))
    });
    found
}
