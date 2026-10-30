#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Reminders arriving, the due date in the create dialog, and the calendar entry form.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::reminders::DueReminder;
use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::date_new;
use studyfrog_core::uid::Uid;
use studyfrog_service::create_more::NewKind;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::{Field, Modal, ModalMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::planner::PlanMsg;
use studyfrog_ui::reminder_ui::{CHECK_EVERY, reminder_text};
use studyfrog_ui::state::Screen;

fn due(minutes_before: u32) -> DueReminder {
    DueReminder {
        session: Uid(1),
        title: "Due: Exam prep".into(),
        date: date_new(2027, 3, 1).unwrap(),
        start_minute: 540,
        minutes_before,
    }
}

#[test]
fn the_calendar_is_checked_on_the_first_tick_and_then_every_so_often() {
    let mut m = open_model();
    let mut checks = Vec::new();
    for tick in 0..=CHECK_EVERY {
        let fx = apply(&mut m, Msg::Tick);
        if fx.contains(&Effect::CheckReminders) {
            checks.push(tick);
        }
        apply(&mut m, Msg::VaultPolled(Ok(None)));
    }
    assert_eq!(checks, [0, CHECK_EVERY]);
    assert!(
        apply(&mut Model::default(), Msg::Tick).is_empty(),
        "no vault, no check"
    );
}

#[test]
fn due_reminders_become_toasts_and_notifications_and_a_click_opens_the_calendar() {
    let mut m = open_model();
    assert!(apply(&mut m, Msg::RemindersDue(Ok(vec![]))).is_empty());
    assert!(m.state.toasts.is_empty());
    let before = m.state.chrome.notifications.len();
    apply(&mut m, Msg::RemindersDue(Ok(vec![due(1440), due(60)])));
    assert_eq!(m.state.toasts.len(), 2);
    assert_eq!(
        reminder_text(&due(1440)),
        "Due: Exam prep (in 1 days, 2027-03-01 09:00)"
    );
    assert_eq!(
        m.state.chrome.notifications.len() - before,
        2,
        "both are in the bell list"
    );
    let fx = apply(&mut m, Msg::Modal(ModalMsg::ToastClicked(0)));
    assert_eq!(m.state.screen, Screen::Calendar);
    assert!(fx.contains(&Effect::LoadSessions));
    apply(&mut m, Msg::RemindersDue(Err("disk".into())));
    assert!(m.state.notice.unwrap().text.contains("disk"));
}

fn create_form(m: &mut Model) {
    apply(m, Msg::Modal(ModalMsg::OpenCreateStack));
    apply(
        m,
        Msg::Modal(ModalMsg::Edit(Field::Title, "Exam prep".into())),
    );
}

#[test]
fn a_stack_due_date_travels_with_the_create_request() {
    let mut m = open_model();
    create_form(&mut m);
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(Field::Due, "2027-03-01".into())),
    );
    apply(&mut m, Msg::Modal(ModalMsg::ToggleReminder(60)));
    apply(&mut m, Msg::Modal(ModalMsg::ToggleReminder(1440))); // the default one goes off
    let fx = apply(&mut m, Msg::Modal(ModalMsg::Submit));
    let [
        Effect::CreateItem {
            kind, due: Some(d), ..
        },
    ] = &fx[..]
    else {
        panic!("{fx:?}")
    };
    assert_eq!(*kind, NewKind::Stack);
    assert_eq!(
        (d.date, d.minute, d.reminders.clone()),
        (date_new(2027, 3, 1).unwrap(), 540, vec![60])
    );
}

#[test]
fn a_wrong_date_is_refused_and_no_date_means_no_deadline() {
    let mut m = open_model();
    create_form(&mut m);
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(Field::Due, "soon".into())),
    );
    assert!(apply(&mut m, Msg::Modal(ModalMsg::Submit)).is_empty());
    let Some(Modal::Create(f)) = &m.state.modal else {
        panic!()
    };
    assert!(f.error.as_deref().unwrap().contains("2026-12-31"));
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(Field::Due, String::new())),
    );
    let fx = apply(&mut m, Msg::Modal(ModalMsg::Submit));
    assert!(matches!(&fx[..], [Effect::CreateItem { due: None, .. }]));
}

#[test]
fn the_create_dialog_offers_a_due_date_and_reminders_for_a_stack_only() {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreateStack));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("Due date (optional)").is_some());
    assert!(
        h.query_by_label("1 day before").is_none(),
        "reminders wait for a date"
    );
    h.get_by_label("Due date (optional)").focus();
    h.get_by_label("Due date (optional)")
        .type_text("2027-03-01");
    h.run();
    h.get_by_label("1 week before").click();
    h.run();
    assert!(h.query_by_label("At the time").is_some());
    assert_all_named(&h, "create dialog with due date");
}

#[test]
fn the_calendar_form_chooses_a_kind_and_reminders() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    apply(
        &mut m,
        Msg::Plan(PlanMsg::Loaded(Ok((
            vec![],
            date_new(2026, 10, 12).unwrap(),
        )))),
    );
    apply(&mut m, Msg::Plan(PlanMsg::New));
    let mut h = harness(m);
    h.run();
    for text in [
        "study",
        "task",
        "appointment",
        "exam",
        "deadline",
        "Remind me",
        "15 minutes before",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    h.get_by_label("exam").click();
    h.run();
    h.get_by_label("1 day before").click();
    h.run();
    let form = h.state().0.state.planner.form.clone().unwrap();
    assert_eq!(form.kind, EventKind::Exam);
    assert_eq!(form.reminders, vec![15, 1440]);
    assert_all_named(&h, "calendar form");
}

#[test]
fn the_date_picker_fills_the_due_date_and_pages_through_months() {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreateStack));
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("2026-01-15").is_none(),
        "closed until asked for"
    );
    h.get_by_label("Pick date").click();
    h.run();
    h.get_by_label("Later month").click();
    h.run();
    assert!(
        h.query_by_label("2026-02-14").is_some(),
        "February is showing"
    );
    h.get_by_label("Earlier month").click();
    h.run();
    h.get_by_label("2026-01-15").click();
    h.run();
    let Some(Modal::Create(f)) = &h.state().0.state.modal else {
        panic!()
    };
    assert_eq!(f.due, "2026-01-15");
    assert!(
        h.query_by_label("2026-01-15").is_none(),
        "the picker closes after a choice"
    );
    assert!(
        h.query_by_label("1 day before").is_some(),
        "reminders appear once there is a date"
    );
}
