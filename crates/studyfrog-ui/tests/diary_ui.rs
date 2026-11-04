#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! UI tests for the Diary screen.

mod common;

use std::collections::BTreeMap;

use common::views::{assert_all_named, harness, open_model};
use studyfrog_core::civil::Date;
use studyfrog_core::diary::RatingValue;
use studyfrog_core::uid::Uid;
use studyfrog_store::diary::StoredDiaryEntry;
use studyfrog_ui::diary_filter::filter_entries;
use studyfrog_ui::diary_msg::DiaryMsg;
use studyfrog_ui::diary_types::{DiaryEffect, DiaryViewMode};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn sample_entry(
    id: u128,
    date: Date,
    title: &str,
    stack: &str,
    perf: i64,
    focus: i64,
) -> StoredDiaryEntry {
    let mut ratings = BTreeMap::new();
    ratings.insert("performance".into(), RatingValue::Int(perf));
    ratings.insert("focus".into(), RatingValue::Int(focus));
    StoredDiaryEntry {
        uid: Uid(id),
        path: format!("diary/{}.md", title.to_lowercase().replace(' ', "-")),
        date,
        title: title.into(),
        study_minutes: Some(40),
        ratings,
        content: format!("Reflections on {title}"),
        mtime: 0,
        runs: Vec::new(),
        stacks: vec![stack.into()],
        items: Vec::new(),
        exams: Vec::new(),
        tags: vec!["test".into()],
    }
}

#[test]
fn navigation_to_diary() {
    let mut m = open_model();
    let fx = apply(&mut m, Msg::Navigate(Screen::Diary));
    assert_eq!(m.state.screen, Screen::Diary);
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::Diary(DiaryEffect::LoadEntries)))
    );

    let h = harness(m);
    assert_all_named(&h, "diary screen");
}

#[test]
fn calendar_date_selection() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Diary));
    apply(
        &mut m,
        Msg::Diary(DiaryMsg::SetViewMode(DiaryViewMode::MonthCalendar)),
    );
    assert_eq!(m.state.diary.view_mode, DiaryViewMode::MonthCalendar);

    let target_date = Date {
        year: 2026,
        month: 11,
        day: 20,
    };
    apply(&mut m, Msg::Diary(DiaryMsg::SelectDate(target_date)));
    assert_eq!(m.state.diary.selected_date, target_date);

    apply(&mut m, Msg::Diary(DiaryMsg::StepMonth(1)));
    assert_eq!(m.state.diary.selected_date.month, 12);
    apply(&mut m, Msg::Diary(DiaryMsg::StepMonth(-1)));
    assert_eq!(m.state.diary.selected_date.month, 11);

    let h = harness(m);
    assert_all_named(&h, "diary month calendar");
}

#[test]
fn filtering_by_stack_and_rating() {
    let d1 = Date {
        year: 2026,
        month: 11,
        day: 1,
    };
    let d2 = Date {
        year: 2026,
        month: 11,
        day: 2,
    };
    let d3 = Date {
        year: 2026,
        month: 11,
        day: 3,
    };

    let e1 = sample_entry(1, d1, "Biochem review", "biochem/metabolism", 4, 5);
    let e2 = sample_entry(2, d2, "Anatomy lab", "anatomy/limbs", 2, 3);
    let e3 = sample_entry(3, d3, "Enzymes kinetics", "biochem/metabolism", 5, 2);

    let entries = vec![e1, e2, e3];

    // Filter by stack
    let biochem_only = filter_entries(&entries, "", Some("biochem"), None, None);
    assert_eq!(biochem_only.len(), 2);
    assert_eq!(biochem_only[0].uid, Uid(3));
    assert_eq!(biochem_only[1].uid, Uid(1));

    // Filter by rating score: performance >= 4
    let high_perf = filter_entries(&entries, "", None, Some("performance"), Some(4));
    assert_eq!(high_perf.len(), 2);

    // Filter by rating score: performance >= 5
    let max_perf = filter_entries(&entries, "", None, Some("performance"), Some(5));
    assert_eq!(max_perf.len(), 1);
    assert_eq!(max_perf[0].uid, Uid(3));

    // Combined filter
    let combined = filter_entries(
        &entries,
        "Enzymes",
        Some("biochem"),
        Some("performance"),
        Some(5),
    );
    assert_eq!(combined.len(), 1);
    assert_eq!(combined[0].uid, Uid(3));
}

#[test]
fn editing_an_entry() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Diary));

    let edit_date = Date {
        year: 2026,
        month: 11,
        day: 4,
    };
    apply(&mut m, Msg::Diary(DiaryMsg::NewEntry(edit_date)));
    assert!(m.state.diary.active_entry.is_some());

    apply(
        &mut m,
        Msg::Diary(DiaryMsg::UpdateDraftTitle("Evening Study".into())),
    );
    apply(&mut m, Msg::Diary(DiaryMsg::UpdateDraftMinutes(Some(45))));
    apply(
        &mut m,
        Msg::Diary(DiaryMsg::UpdateDraftStacks("biochem".into())),
    );
    apply(
        &mut m,
        Msg::Diary(DiaryMsg::UpdateDraftTags("evening, enzymes".into())),
    );
    apply(
        &mut m,
        Msg::Diary(DiaryMsg::UpdateDraftRating(
            "focus".into(),
            Some(RatingValue::Int(4)),
        )),
    );
    apply(
        &mut m,
        Msg::Diary(DiaryMsg::UpdateDraftContent(
            "Worked through kinetics problems.".into(),
        )),
    );

    let draft = m.state.diary.active_entry.as_ref().unwrap();
    assert_eq!(draft.title, "Evening Study");
    assert_eq!(draft.study_minutes, Some(45));
    assert_eq!(draft.stacks, "biochem");
    assert_eq!(draft.content, "Worked through kinetics problems.");
    assert_eq!(draft.ratings.get("focus"), Some(&RatingValue::Int(4)));
    assert!(draft.is_dirty);

    let h = harness(m);
    assert_all_named(&h, "diary active editor");
}
