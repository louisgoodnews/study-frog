//! The typed search of the library (`docs/specs/query.md`): the syntax tree and the errors. Parsing is in
//! [`crate::query_parse`], evaluation in [`crate::query_eval`].

use crate::event::Difficulty;
use crate::plan::ItemKind;
use crate::priority::Priority;
use crate::status::Status;

/// A field of an item that a query can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// Title and body together.
    Text,
    /// The title.
    Title,
    /// Note, card or question.
    Type,
    /// The effective status.
    Status,
    /// The stack, with the stacks below it.
    Stack,
    /// The subjects of the item's stack.
    Subject,
    /// The teachers of those subjects.
    Teacher,
    /// A tag.
    Tag,
    /// The effective priority.
    Priority,
    /// The effective difficulty.
    Difficulty,
    /// Familiarity, 0 to 1.
    Familiarity,
    /// The next review.
    Due,
    /// How many reviews the item has.
    Reviews,
    /// When it was created.
    Created,
    /// When it was last updated.
    Updated,
    /// The file's path in the vault.
    Path,
    /// The item's key (`FLASHCARD_500`).
    Key,
}

/// How a comparison compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `=`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `~`
    Contains,
    /// `!~`
    NotContains,
}

/// A unit of a relative date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateUnit {
    /// Days.
    Day,
    /// Weeks.
    Week,
    /// Months.
    Month,
    /// Years.
    Year,
}

/// A date in a query: a fixed day, or a count of units from the day the query runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateVal {
    /// A day, counted from 1970-01-01.
    Day(i64),
    /// `7d`, `-14d`, `2w`, `1m`, `1y`; `today` is `0d`.
    Relative(i64, DateUnit),
}

/// A value, already checked against its field.
#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    /// Free text (compared without regard to case).
    Text(String),
    /// A kind.
    Kind(ItemKind),
    /// A status.
    Status(Status),
    /// A priority.
    Priority(Priority),
    /// A difficulty.
    Difficulty(Difficulty),
    /// A number (familiarity).
    Num(f64),
    /// A whole number.
    Int(i64),
    /// A date.
    Date(DateVal),
}

/// A condition.
#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    /// `field op value`.
    Cmp(Field, Op, Val),
    /// `field IN (...)`, negated for `NOT IN`.
    In(Field, Vec<Val>, bool),
    /// `field IS EMPTY`, negated for `IS NOT EMPTY`.
    Empty(Field, bool),
    /// All of them.
    And(Vec<Cond>),
    /// Any of them.
    Or(Vec<Cond>),
    /// None of it.
    Not(Box<Cond>),
}

/// One key of `ORDER BY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Order {
    /// The field.
    pub field: Field,
    /// Largest first.
    pub descending: bool,
}

/// A whole query.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Query {
    /// What items must satisfy; `None` matches every item.
    pub cond: Option<Cond>,
    /// How to sort, at most three keys.
    pub order: Vec<Order>,
}

/// Why a query could not be read, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError {
    /// The column the message is about, counted from 1.
    pub column: usize,
    /// The message, in words.
    pub message: String,
}

/// Longest query accepted, in characters.
pub const MAX_QUERY_CHARS: usize = 2000;
/// Deepest nesting of parentheses accepted.
pub const MAX_DEPTH: usize = 20;
/// Most `ORDER BY` keys accepted.
pub const MAX_ORDER_KEYS: usize = 3;
