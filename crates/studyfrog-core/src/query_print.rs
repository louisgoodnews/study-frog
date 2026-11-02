//! A query written out again in the language, so that the Standard mode can show what it filters by and a saved
//! filter can be stored as text. `parse_query(print_query(q))` gives back `q`.

use crate::civil::{date_from_days, format_date};
use crate::event::difficulty_name;
use crate::plan::item_kind_name;
use crate::priority::priority_name;
use crate::query::{Cond, DateUnit, DateVal, Order, Query, Val};
use crate::query_fields::{field_name, op_symbol};
use crate::status::status_name;

fn word_like(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '%'))
}

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn date(d: DateVal) -> String {
    match d {
        DateVal::Day(day) => format_date(date_from_days(day)),
        DateVal::Relative(0, DateUnit::Day) => "today".to_owned(),
        DateVal::Relative(n, unit) => {
            let u = match unit {
                DateUnit::Day => 'd',
                DateUnit::Week => 'w',
                DateUnit::Month => 'm',
                DateUnit::Year => 'y',
            };
            format!("{n}{u}")
        }
    }
}

fn value(v: &Val) -> String {
    match v {
        Val::Text(s) if word_like(s) && s.parse::<f64>().is_err() => s.clone(),
        Val::Text(s) => quoted(s),
        Val::Kind(k) => item_kind_name(*k).to_owned(),
        Val::Status(s) => status_name(*s).to_owned(),
        Val::Priority(p) => priority_name(*p).to_owned(),
        Val::Difficulty(d) => difficulty_name(*d).to_owned(),
        Val::Num(n) => format!("{n}"),
        Val::Int(n) => n.to_string(),
        Val::Date(d) => date(*d),
    }
}

fn joined(parts: &[Cond], word: &str, wrap: bool) -> String {
    let texts: Vec<String> = parts.iter().map(|p| cond(p, true)).collect();
    let all = texts.join(&format!(" {word} "));
    if wrap { format!("({all})") } else { all }
}

/// `inner` is true when the condition sits inside another and so needs parentheses if it is an `OR`.
fn cond(c: &Cond, inner: bool) -> String {
    match c {
        Cond::Cmp(f, op, v) => format!("{} {} {}", field_name(*f), op_symbol(*op), value(v)),
        Cond::In(f, vals, neg) => {
            let list: Vec<String> = vals.iter().map(value).collect();
            format!(
                "{} {}IN ({})",
                field_name(*f),
                if *neg { "NOT " } else { "" },
                list.join(", ")
            )
        }
        Cond::Empty(f, neg) => format!(
            "{} IS {}EMPTY",
            field_name(*f),
            if *neg { "NOT " } else { "" }
        ),
        Cond::And(parts) => joined(parts, "AND", false),
        Cond::Or(parts) => joined(parts, "OR", inner),
        Cond::Not(inner_cond) => match **inner_cond {
            Cond::And(_) | Cond::Or(_) => format!("NOT ({})", cond(inner_cond, false)),
            _ => format!("NOT {}", cond(inner_cond, true)),
        },
    }
}

fn order(o: &Order) -> String {
    format!(
        "{}{}",
        field_name(o.field),
        if o.descending { " DESC" } else { "" }
    )
}

/// The query as text.
pub fn print_query(q: &Query) -> String {
    let mut out = q.cond.as_ref().map(|c| cond(c, false)).unwrap_or_default();
    if !q.order.is_empty() {
        let keys: Vec<String> = q.order.iter().map(order).collect();
        out.push_str(&format!(
            "{}ORDER BY {}",
            if out.is_empty() { "" } else { " " },
            keys.join(", ")
        ));
    }
    out
}
