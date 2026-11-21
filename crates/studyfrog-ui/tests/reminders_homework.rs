#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Homework deadline reminders arriving while the app is open (STORY-0166).

mod common;

use common::views::open_model;
use studyfrog_calendar::reminders::DueAll;
use studyfrog_core::civil::date_new;
use studyfrog_core::uid::Uid;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

#[test]
fn homework_deadlines_become_toasts_and_notifications_too() {
    use studyfrog_calendar::homework_reminders::DueHomeworkReminder;
    use studyfrog_core::civil::LocalTime;
    use studyfrog_ui::reminder_ui::homework_text;
    let r = DueHomeworkReminder {
        item: Uid(7),
        title: "Essay 2".into(),
        due: LocalTime {
            date: date_new(2026, 12, 2).unwrap(),
            minute: 540,
        },
        lead_min: 120,
    };
    assert_eq!(homework_text(&r), "Essay 2 (in 2 h, 2026-12-02 09:00)");
    let mut m = open_model();
    let before = m.state.chrome.notifications.len();
    apply(&mut m, Msg::RemindersDue(Ok(DueAll::default())));
    assert!(m.state.toasts.is_empty());
    let homework = vec![r];
    apply(
        &mut m,
        Msg::RemindersDue(Ok(DueAll {
            sessions: Vec::new(),
            homework,
        })),
    );
    assert_eq!(m.state.toasts.len(), 1);
    assert_eq!(m.state.chrome.notifications.len() - before, 1);
    apply(&mut m, Msg::RemindersDue(Err("disk".into())));
    assert!(m.state.notice.unwrap().text.contains("disk"));
}
