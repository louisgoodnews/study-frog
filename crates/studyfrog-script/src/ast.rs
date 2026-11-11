//! Abstract Syntax Tree (AST) node representations for FrogScript.

use crate::error::Span;

/// A complete FrogScript program consisting of sequential statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Ordered list of top-level statements.
    pub statements: Vec<Stmt>,
}

/// A statement in FrogScript.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Standalone expression (e.g. function call, pipeline, literal).
    Expr(Expr),
    /// Variable assignment (`x = 10` or `cards = from stack ...`).
    Assign {
        /// Assigned variable identifier.
        target: String,
        /// Evaluated value expression.
        value: Expr,
        /// Source location span.
        span: Span,
    },
    /// Conditional branching (`if ... elif ... else ...`).
    If {
        /// Primary if condition.
        condition: Expr,
        /// Executed when condition is truthy.
        then_body: Vec<Stmt>,
        /// Optional elif condition-block pairs.
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        /// Optional else fallback block.
        else_body: Option<Vec<Stmt>>,
        /// Source location span.
        span: Span,
    },
    /// For-in iteration loop.
    For {
        /// Loop item variable name.
        item_var: String,
        /// Optional index variable name.
        index_var: Option<String>,
        /// Expression evaluating to an iterable collection.
        iter: Expr,
        /// Body statements executed per item.
        body: Vec<Stmt>,
        /// Source location span.
        span: Span,
    },
    /// Custom function declaration.
    Def {
        /// Function name.
        name: String,
        /// Parameter names.
        params: Vec<String>,
        /// Body statements.
        body: Vec<Stmt>,
        /// Source location span.
        span: Span,
    },
    /// Return statement from a function or block.
    Return {
        /// Returned expression value, or None for void return.
        value: Option<Expr>,
        /// Source location span.
        span: Span,
    },
}

/// An expression producing a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal constant value.
    Literal(Literal, Span),
    /// Identifier reference.
    Ident(String, Span),
    /// Binary arithmetic, comparison, or logical operation.
    Binary {
        /// Operator.
        op: BinOp,
        /// Left-hand operand.
        left: Box<Expr>,
        /// Right-hand operand.
        right: Box<Expr>,
        /// Source location span.
        span: Span,
    },
    /// Unary operation (negation, logical not).
    Unary {
        /// Operator.
        op: UnOp,
        /// Target expression.
        expr: Box<Expr>,
        /// Source location span.
        span: Span,
    },
    /// Pipeline stream transformation (`source | stage1 | stage2`).
    Pipeline {
        /// Initial source collection or value.
        source: Box<Expr>,
        /// Chained pipeline transformation stages.
        stages: Vec<PipeStage>,
        /// Source location span.
        span: Span,
    },
    /// `from stack ...` or `from vault` query expression.
    From {
        /// Target stack name or wildcard, or None for whole vault.
        target: Option<Box<Expr>>,
        /// Source location span.
        span: Span,
    },
    /// Closure expression (`{ item -> action }` or multi-line block).
    Closure {
        /// Parameter names.
        params: Vec<String>,
        /// Body statements or expressions.
        body: Vec<Stmt>,
        /// Source location span.
        span: Span,
    },
    /// Function or method invocation (`func(a, b)`).
    Call {
        /// Callee expression.
        callee: Box<Expr>,
        /// Argument expressions.
        args: Vec<Expr>,
        /// Source location span.
        span: Span,
    },
    /// Field or property access (`obj.field`).
    FieldAccess {
        /// Target object expression.
        target: Box<Expr>,
        /// Field identifier.
        field: String,
        /// Source location span.
        span: Span,
    },
    /// List literal `[a, b, c]`.
    List(Vec<Expr>, Span),
    /// Map / dictionary literal `{k: v}`.
    Dict(Vec<(Expr, Expr)>, Span),
}

/// Individual stages in a pipeline expression (`| ...`).
#[derive(Debug, Clone, PartialEq)]
pub enum PipeStage {
    /// Filtering stage `where <condition>`.
    Where(Expr),
    /// Field mutation stage `set <field> <value>`.
    Set {
        /// Field name to mutate.
        field: String,
        /// New value expression.
        value: Expr,
    },
    /// Tag addition `tag <tag_name>`.
    Tag(Expr),
    /// Tag removal `untag <tag_name>`.
    Untag(Expr),
    /// Sort ordering `sort <field> [asc|desc]`.
    Sort {
        /// Field to sort by.
        field: String,
        /// True if descending sort order.
        descending: bool,
    },
    /// Result limit `limit <count>`.
    Limit(Expr),
    /// Field projection `select a, b, c`.
    Select(Vec<String>),
    /// Export operation `export <kind> <path>`.
    Export {
        /// Target export format / entity kind.
        kind: String,
        /// File destination path expression.
        path: Expr,
    },
    /// Direct closure applied to piped items `{ x -> ... }`.
    Closure(Expr),
    /// Custom function pipe stage `| my_func(...)`.
    Custom(Expr),
}

/// Literal values in FrogScript.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer number.
    Int(i64),
    /// Floating point number.
    Float(f64),
    /// Duration in seconds.
    DurationSecs(i64),
    /// Quoted text string.
    Str(String),
    /// Formatted interpolated string.
    FormatStr(String),
    /// Boolean value.
    Bool(bool),
    /// Null / None value.
    Null,
    /// Wildcard `*`.
    Wildcard,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// Addition `+`
    Add,
    /// Subtraction `-`
    Sub,
    /// Multiplication `*`
    Mul,
    /// Division `/`
    Div,
    /// Modulo `%`
    Mod,
    /// Equal `==`
    Eq,
    /// Not equal `!=`
    NotEq,
    /// Less than `<`
    Lt,
    /// Less than or equal `<=`
    LtEq,
    /// Greater than `>`
    Gt,
    /// Greater than or equal `>=`
    GtEq,
    /// Logical conjunction `and`
    And,
    /// Logical disjunction `or`
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// Arithmetic negation `-`
    Neg,
    /// Logical inversion `not`
    Not,
}
