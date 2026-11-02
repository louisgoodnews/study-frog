//! The parser of one condition: `field op value`, `field IN (...)` and `field IS EMPTY`.

use crate::query::{Cond, Field, Op, Val};
use crate::query_cursor::{Cursor, Parsed, fail, is_sym, is_word, missing_at, peek, shown, take};
use crate::query_fields::{
    FIELDS, can_be_empty, describe_ops, field_by_name, field_name, op_symbol, ops_allowed, suggest,
    takes_in,
};
use crate::query_lex::{Tok, Token};
use crate::query_values::value_of;

pub(crate) fn expect_value(c: &mut Cursor, field: Field, after: &Token) -> Parsed<Val> {
    match take(c) {
        Some(t) if !matches!(t.tok, Tok::Sym(_)) => value_of(field, t),
        Some(t) => fail(
            t.col,
            format!(
                "Expected a value after `{}` at column {}",
                shown(after),
                t.col
            ),
        ),
        None => fail(
            after.end,
            format!(
                "Expected a value after `{}` at column {}",
                shown(after),
                after.end
            ),
        ),
    }
}

pub(crate) fn field_of(c: &mut Cursor, after: Option<&Token>) -> Parsed<(Field, Token)> {
    let Some(t) = take(c) else {
        let (col, what) = after.map_or((1, String::new()), |a| {
            (a.end, format!(" after `{}`", shown(a)))
        });
        return fail(col, format!("Expected a field{what} at column {col}"));
    };
    let name = match &t.tok {
        Tok::Word(w) => w,
        _ => return fail(t.col, format!("Expected a field at column {}", t.col)),
    };
    match field_by_name(name) {
        Some(f) => Ok((f, t.clone())),
        None => {
            let hint = suggest(name).map_or_else(
                || format!("The fields are {}", FIELDS.map(|(_, n)| n).join(", ")),
                |s| format!("Did you mean `{s}`?"),
            );
            fail(
                t.col,
                format!("Unknown field `{name}` at column {}. {hint}", t.col),
            )
        }
    }
}

pub(crate) fn comparison_op(sym: &str) -> Option<Op> {
    Some(match sym {
        "=" => Op::Eq,
        "!=" => Op::Ne,
        "<" => Op::Lt,
        "<=" => Op::Le,
        ">" => Op::Gt,
        ">=" => Op::Ge,
        "~" => Op::Contains,
        "!~" => Op::NotContains,
        _ => return None,
    })
}

pub(crate) fn in_list(c: &mut Cursor, field: Field, negated: bool, after: &Token) -> Parsed<Cond> {
    if !takes_in(field) {
        return fail(
            after.col,
            format!(
                "`IN` does not apply to `{}`; use {} (column {})",
                field_name(field),
                describe_ops(field),
                after.col
            ),
        );
    }
    let open = take(c);
    if !is_sym(open, "(") {
        let col = open.map_or(after.end, |t| t.col);
        return fail(col, format!("Expected `(` after IN at column {col}"));
    }
    let mut values = Vec::new();
    loop {
        let before = take(c).cloned();
        let Some(t) = before else {
            return fail(
                missing_at(c),
                format!("Expected `)` at column {}", missing_at(c)),
            );
        };
        values.push(match &t.tok {
            Tok::Sym(_) => return fail(t.col, format!("Expected a value at column {}", t.col)),
            _ => value_of(field, &t)?,
        });
        match take(c) {
            Some(n) if is_sym(Some(n), ",") => {}
            Some(n) if is_sym(Some(n), ")") => return Ok(Cond::In(field, values, negated)),
            Some(n) => return fail(n.col, format!("Expected `,` or `)` at column {}", n.col)),
            None => {
                return fail(
                    missing_at(c),
                    format!("Expected `)` at column {}", missing_at(c)),
                );
            }
        }
    }
}

pub(crate) fn is_empty(c: &mut Cursor, field: Field, is_tok: &Token) -> Parsed<Cond> {
    let negated = is_word(peek(c), "NOT");
    c.pos += usize::from(negated);
    match take(c) {
        Some(t) if is_word(Some(t), "EMPTY") => {
            if can_be_empty(field) {
                Ok(Cond::Empty(field, negated))
            } else {
                fail(
                    is_tok.col,
                    format!(
                        "`{}` is never empty (column {})",
                        field_name(field),
                        is_tok.col
                    ),
                )
            }
        }
        other => {
            let col = other.map_or(missing_at(c), |t| t.col);
            fail(col, format!("Expected EMPTY after IS at column {col}"))
        }
    }
}

pub(crate) fn comparison(c: &mut Cursor) -> Parsed<Cond> {
    let (field, ft) = field_of(c, None)?;
    let Some(op_tok) = take(c) else {
        return fail(
            ft.end,
            format!(
                "Expected an operator after `{}` at column {}",
                field_name(field),
                ft.end
            ),
        );
    };
    if is_word(Some(op_tok), "IN") {
        return in_list(c, field, false, op_tok);
    }
    if is_word(Some(op_tok), "NOT") && is_word(peek(c), "IN") {
        c.pos += 1;
        return in_list(c, field, true, op_tok);
    }
    if is_word(Some(op_tok), "IS") {
        return is_empty(c, field, op_tok);
    }
    let op = match &op_tok.tok {
        Tok::Sym(s) => comparison_op(s),
        _ => None,
    };
    let Some(op) = op else {
        let ops = describe_ops(field);
        return fail(
            op_tok.col,
            format!(
                "Expected an operator after `{}` at column {}; `{}` takes {ops}",
                field_name(field),
                op_tok.col,
                field_name(field)
            ),
        );
    };
    if !ops_allowed(field).contains(&op) {
        let col = op_tok.col;
        return fail(
            col,
            format!(
                "`{}` does not apply to `{}`; use {} (column {col})",
                op_symbol(op),
                field_name(field),
                describe_ops(field)
            ),
        );
    }
    Ok(Cond::Cmp(field, op, expect_value(c, field, op_tok)?))
}
