#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Running queries over items: what matches, in which order, and what a missing value does.

use studyfrog_core::civil::{date_to_days, parse_date};
use studyfrog_core::event::Difficulty;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::Priority;
use studyfrog_core::query_eval::{Row, run};
use studyfrog_core::query_parse::parse_query;
use studyfrog_core::status::Status;

fn day(text: &str) -> i64 {
    date_to_days(parse_date(text).unwrap())
}

const TODAY: &str = "2026-11-10";

fn row(path: &str, title: &str) -> Row {
    Row {
        path: path.into(),
        title: title.into(),
        body: String::new(),
        kind: ItemKind::Card,
        status: Status::New,
        stacks: vec!["french".into(), "French".into()],
        subjects: Vec::new(),
        teachers: Vec::new(),
        also_in: Vec::new(),
        tags: Vec::new(),
        priority: Priority::Medium,
        difficulty: None,
        familiarity: None,
        due_day: None,
        reviews: 0,
        created_day: Some(day("2026-01-01")),
        updated_day: Some(day("2026-11-01")),
    }
}

fn rows() -> Vec<Row> {
    let mut a = row("french/a.md", "Bonjour");
    (a.status, a.familiarity, a.due_day, a.reviews) =
        (Status::Learning, Some(0.3), Some(day("2026-11-12")), 4);
    a.tags = vec!["exam".into(), "Greeting".into()];
    a.body = "Hello, good morning".into();
    let mut b = row("french/verbs/b.md", "Être");
    (b.status, b.familiarity, b.due_day, b.reviews) =
        (Status::Reviewing, Some(0.9), Some(day("2026-12-24")), 11);
    b.stacks = vec![
        "french".into(),
        "French".into(),
        "french/verbs".into(),
        "Verbs".into(),
    ];
    (b.subjects, b.teachers, b.difficulty) = (
        vec!["Languages".into()],
        vec!["Prof. Dr. Künkelmann, Harry".into()],
        Some(Difficulty::Difficult),
    );
    let mut c = row("maths/c.md", "Prime numbers");
    (c.kind, c.status, c.priority) = (ItemKind::Note, Status::Done, Priority::High);
    c.stacks = vec!["maths".into(), "Mathematics".into()];
    c.subjects = vec!["Maths".into()];
    c.body = "A prime has two divisors.".into();
    let mut d = row("maths/d.md", "Derivatives");
    (d.kind, d.priority, d.difficulty) = (
        ItemKind::Question,
        Priority::Highest,
        Some(Difficulty::Easy),
    );
    d.stacks = vec!["maths".into(), "Mathematics".into()];
    d.updated_day = None;
    d.tags = vec!["Exam".into()];
    vec![a, b, c, d]
}

fn found(q: &str) -> Vec<String> {
    let all = rows();
    run(&parse_query(q).unwrap(), &all, day(TODAY))
        .into_iter()
        .map(|i| all[i].title.clone())
        .collect()
}

#[test]
fn comparisons_on_text_enumerations_and_numbers() {
    assert_eq!(
        found(""),
        ["Bonjour", "Être", "Prime numbers", "Derivatives"]
    );
    assert_eq!(found("type = card"), ["Bonjour", "Être"]);
    assert_eq!(
        found("type IN (note, question)"),
        ["Prime numbers", "Derivatives"]
    );
    assert_eq!(found("status = learning"), ["Bonjour"]);
    assert_eq!(
        found("status NOT IN (new, learning)"),
        ["Être", "Prime numbers"]
    );
    assert_eq!(found("familiarity < 0.5"), ["Bonjour"]);
    assert_eq!(found("familiarity >= 90%"), ["Être"]);
    assert_eq!(found("reviews >= 4 AND reviews < 11"), ["Bonjour"]);
    assert_eq!(found("priority >= high"), ["Prime numbers", "Derivatives"]);
    assert_eq!(found("priority = medium"), ["Bonjour", "Être"]);
}

#[test]
fn text_is_compared_without_regard_to_case_and_searches_title_and_body() {
    assert_eq!(found("title = bonjour"), ["Bonjour"]);
    assert_eq!(found("title ~ \"NUMB\""), ["Prime numbers"]);
    assert_eq!(found("text ~ divisors"), ["Prime numbers"]);
    assert_eq!(
        found("text ~ \"good morning\" OR text ~ derivatives"),
        ["Bonjour", "Derivatives"]
    );
    assert_eq!(found("title !~ e"), ["Bonjour"]);
    assert_eq!(found("path ~ maths/"), ["Prime numbers", "Derivatives"]);
    assert_eq!(found("tag = exam"), ["Bonjour", "Derivatives"]);
    assert_eq!(found("tag != exam"), ["Être", "Prime numbers"]);
}

#[test]
fn a_stack_includes_the_stacks_below_it_and_is_named_by_folder_or_title() {
    assert_eq!(found("stack = french"), ["Bonjour", "Être"]);
    assert_eq!(found("stack = \"french/verbs\""), ["Être"]);
    assert_eq!(found("stack = Verbs"), ["Être"]);
    assert_eq!(
        found("stack = mathematics"),
        ["Prime numbers", "Derivatives"]
    );
    assert_eq!(
        found("stack IN (verbs, maths)"),
        ["Être", "Prime numbers", "Derivatives"]
    );
}

#[test]
fn subjects_and_teachers_with_commas_are_matched_whole() {
    assert_eq!(found("teacher = \"Prof. Dr. Künkelmann, Harry\""), ["Être"]);
    assert_eq!(
        found("subject = maths OR subject = languages"),
        ["Être", "Prime numbers"]
    );
    assert_eq!(found("subject IS EMPTY"), ["Bonjour", "Derivatives"]);
    assert_eq!(found("teacher IS NOT EMPTY"), ["Être"]);
}

#[test]
fn dates_are_fixed_or_counted_from_today_and_missing_ones_match_only_not_equal() {
    assert_eq!(found("due <= 7d"), ["Bonjour"]);
    assert_eq!(found("due > today AND due <= 2m"), ["Bonjour", "Être"]);
    assert_eq!(found("due = 2026-12-24"), ["Être"]);
    assert_eq!(found("due IS EMPTY"), ["Prime numbers", "Derivatives"]);
    assert_eq!(
        found("due != 2026-11-12"),
        ["Être", "Prime numbers", "Derivatives"]
    );
    assert_eq!(
        found("updated >= -14d"),
        ["Bonjour", "Être", "Prime numbers"]
    );
    assert_eq!(found("updated IS EMPTY"), ["Derivatives"]);
    assert_eq!(
        found("created < -6m"),
        ["Bonjour", "Être", "Prime numbers", "Derivatives"]
    );
    assert_eq!(
        found("created >= -1y AND created < 1w"),
        found("created < 1w")
    );
}

#[test]
fn difficulty_is_ordered_and_a_missing_one_matches_only_not_equal() {
    assert_eq!(found("difficulty = difficult"), ["Être"]);
    assert_eq!(found("difficulty < difficult"), ["Derivatives"]);
    assert_eq!(
        found("difficulty != easy"),
        ["Bonjour", "Être", "Prime numbers"]
    );
    assert_eq!(found("difficulty IS EMPTY"), ["Bonjour", "Prime numbers"]);
    assert_eq!(
        found("familiarity IS EMPTY"),
        ["Prime numbers", "Derivatives"]
    );
}

#[test]
fn not_and_parentheses_and_or_combine() {
    assert_eq!(found("NOT type = card"), ["Prime numbers", "Derivatives"]);
    assert_eq!(
        found("NOT (type = card OR priority = highest)"),
        ["Prime numbers"]
    );
    assert_eq!(
        found("(type = note OR type = question) AND NOT tag = exam"),
        ["Prime numbers"]
    );
    assert_eq!(
        found("status = done OR status = learning AND type = note"),
        ["Prime numbers"]
    );
}

#[test]
fn results_are_sorted_by_the_keys_with_missing_values_last_and_ties_by_path() {
    assert_eq!(
        found("ORDER BY familiarity"),
        ["Bonjour", "Être", "Prime numbers", "Derivatives"]
    );
    assert_eq!(
        found("ORDER BY familiarity DESC"),
        ["Être", "Bonjour", "Prime numbers", "Derivatives"]
    );
    assert_eq!(
        found("ORDER BY due DESC, title"),
        ["Être", "Bonjour", "Derivatives", "Prime numbers"]
    );
    assert_eq!(
        found("ORDER BY priority DESC, title"),
        ["Derivatives", "Prime numbers", "Bonjour", "Être"]
    );
    assert_eq!(
        found("ORDER BY title"),
        ["Bonjour", "Derivatives", "Prime numbers", "Être"]
            .map(String::from)
            .to_vec()
    );
    assert_eq!(
        found("type = card ORDER BY reviews DESC"),
        ["Être", "Bonjour"]
    );
    assert_eq!(
        found("ORDER BY type, status DESC"),
        ["Prime numbers", "Être", "Bonjour", "Derivatives"]
    );
}
