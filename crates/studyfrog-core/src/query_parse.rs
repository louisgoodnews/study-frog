//! The parser of the query language (`docs/specs/query.md`): a hand-written recursive descent over the tokens.
//! Every error says which column it is about and what was expected.

use crate::query::{Cond, MAX_DEPTH, MAX_ORDER_KEYS, MAX_QUERY_CHARS, Order, Query, QueryError};
use crate::query_cond::{comparison, field_of};
use crate::query_cursor::{Cursor, Parsed, fail, is_sym, is_word, missing_at, peek, shown, take};
use crate::query_fields::{can_sort, field_name};
use crate::query_lex::{Token, lex};

fn nest(c: &mut Cursor, at: &Token) -> Parsed<()> {
    c.depth += 1;
    if c.depth > MAX_DEPTH {
        return fail(
            at.col,
            format!(
                "Too many nested conditions at column {} (the most is {MAX_DEPTH})",
                at.col
            ),
        );
    }
    Ok(())
}

fn primary(c: &mut Cursor) -> Parsed<Cond> {
    if let Some(open) = peek(c).filter(|t| is_sym(Some(t), "(")) {
        c.pos += 1;
        nest(c, open)?;
        let inner = or(c)?;
        if !is_sym(take(c), ")") {
            return fail(
                open.col,
                format!("Missing `)` for the `(` at column {}", open.col),
            );
        }
        c.depth -= 1;
        return Ok(inner);
    }
    comparison(c)
}

fn not(c: &mut Cursor) -> Parsed<Cond> {
    let Some(word) = peek(c).filter(|t| is_word(Some(t), "NOT")) else {
        return primary(c);
    };
    c.pos += 1;
    nest(c, word)?;
    if peek(c).is_none() {
        return fail(
            word.end,
            format!("Expected a condition after NOT at column {}", word.end),
        );
    }
    let inner = not(c)?;
    c.depth -= 1;
    Ok(Cond::Not(Box::new(inner)))
}

/// The parser of one level: the next-tighter level.
type Level = fn(&mut Cursor) -> Parsed<Cond>;

fn joined(
    c: &mut Cursor,
    (word, next): (&str, Level),
    build: fn(Vec<Cond>) -> Cond,
) -> Parsed<Cond> {
    let mut parts = vec![next(c)?];
    while let Some(kw) = peek(c).filter(|t| is_word(Some(t), word)) {
        c.pos += 1;
        if peek(c).is_none() {
            return fail(
                kw.end,
                format!("Expected a condition after {word} at column {}", kw.end),
            );
        }
        parts.push(next(c)?);
    }
    Ok(if parts.len() == 1 {
        parts.remove(0)
    } else {
        build(parts)
    })
}

fn and(c: &mut Cursor) -> Parsed<Cond> {
    joined(c, ("AND", not), Cond::And)
}

fn or(c: &mut Cursor) -> Parsed<Cond> {
    joined(c, ("OR", and), Cond::Or)
}

fn order_by(c: &mut Cursor) -> Parsed<Vec<Order>> {
    let order = take(c).cloned();
    let Some(order) = order else {
        return Ok(Vec::new());
    };
    if !is_word(peek(c), "BY") {
        let col = peek(c).map_or(order.end, |t| t.col);
        return fail(col, format!("Expected BY after ORDER at column {col}"));
    }
    let by = take(c).cloned().unwrap_or(order);
    let mut keys: Vec<Order> = Vec::new();
    loop {
        let (field, ft) = field_of(c, Some(&by))?;
        if !can_sort(field) {
            return fail(
                ft.col,
                format!(
                    "A result cannot be sorted by `{}` (column {})",
                    field_name(field),
                    ft.col
                ),
            );
        }
        let descending = is_word(peek(c), "DESC");
        c.pos += usize::from(descending || is_word(peek(c), "ASC"));
        keys.push(Order { field, descending });
        if keys.len() > MAX_ORDER_KEYS {
            return fail(
                ft.col,
                format!(
                    "ORDER BY takes at most {MAX_ORDER_KEYS} keys (column {})",
                    ft.col
                ),
            );
        }
        if !is_sym(peek(c), ",") {
            return Ok(keys);
        }
        c.pos += 1;
        by_after_comma(c)?;
    }
}

fn by_after_comma(c: &Cursor) -> Parsed<()> {
    match peek(c) {
        Some(_) => Ok(()),
        None => fail(
            missing_at(c),
            format!("Expected a field after `,` at column {}", missing_at(c)),
        ),
    }
}

/// Read a query. An empty query matches every item.
pub fn parse_query(text: &str) -> Result<Query, QueryError> {
    if text.chars().count() > MAX_QUERY_CHARS {
        return fail(
            MAX_QUERY_CHARS + 1,
            format!("A query is at most {MAX_QUERY_CHARS} characters"),
        );
    }
    let toks = lex(text)?;
    let mut c = Cursor {
        toks: &toks,
        pos: 0,
        depth: 0,
    };
    let starts_with_order = is_word(peek(&c), "ORDER");
    let cond = if toks.is_empty() || starts_with_order {
        None
    } else {
        Some(or(&mut c)?)
    };
    let order = if is_word(peek(&c), "ORDER") {
        order_by(&mut c)?
    } else {
        Vec::new()
    };
    if let Some(t) = peek(&c) {
        let msg = if is_sym(Some(t), ")") {
            format!(
                "Unexpected `)` at column {}: there is no `(` to close",
                t.col
            )
        } else {
            format!(
                "Unexpected `{}` at column {}. Expected AND, OR or ORDER BY",
                shown(t),
                t.col
            )
        };
        return fail(t.col, msg);
    }
    Ok(Query { cond, order })
}
