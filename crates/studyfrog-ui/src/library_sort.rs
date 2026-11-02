//! Sorting and the column choice of the library list. Pure.

use studyfrog_core::query::{Field, MAX_ORDER_KEYS, Order};

use crate::library_columns::{COLUMNS, Column};

pub(crate) fn toggle(columns: &[Column], c: Column) -> Vec<Column> {
    if c == Column::Title {
        return columns.to_vec();
    }
    let mut next: Vec<Column> = columns.to_vec();
    match next.iter().position(|x| *x == c) {
        Some(i) => {
            next.remove(i);
        }
        None => next.push(c),
    }
    COLUMNS
        .iter()
        .map(|(x, _, _)| *x)
        .filter(|x| next.contains(x))
        .collect()
}

pub(crate) fn sorted_by(sort: &[Order], field: Field, add: bool) -> Vec<Order> {
    let current = sort.iter().find(|o| o.field == field).copied();
    let flipped = Order {
        field,
        descending: current.is_some_and(|o| !o.descending),
    };
    if !add {
        return vec![flipped];
    }
    let mut next: Vec<Order> = sort.iter().filter(|o| o.field != field).copied().collect();
    next.push(flipped);
    let start = next.len().saturating_sub(MAX_ORDER_KEYS);
    next.split_off(start)
}
