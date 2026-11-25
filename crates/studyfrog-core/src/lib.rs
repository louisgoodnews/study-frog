//! Domain types (Stack, Note, Card, Question, Run, ...) as pure data. No I/O, no behaviour blocks; the root of the dependency graph.

pub mod achievement;
pub mod appearance;
pub mod arena;
pub mod choice_grading;
pub mod civil;
pub mod classroom;
pub mod classroom_ops;
pub mod classroom_status;
pub mod classroom_work;
pub mod comment;
pub mod count;
pub mod diary;
pub mod event;
pub mod exam;
pub mod idx;
pub mod item_color;
pub mod item_key;
pub mod items;
pub mod library;
pub mod memory;
pub mod meta;
pub mod pacing;
pub mod plan;
pub mod plan_names;
pub mod priority;
pub mod query;
pub mod query_complete;
pub mod query_cond;
pub mod query_cursor;
pub mod query_eval;
pub mod query_fields;
pub mod query_lex;
pub mod query_parse;
pub mod query_print;
pub mod query_values;
pub mod review;
pub mod run;
pub mod scheduler;
pub mod settings;
pub mod stack;
pub mod status;
pub mod time;
pub mod uid;
pub mod views;

#[cfg(test)]
mod tests;
