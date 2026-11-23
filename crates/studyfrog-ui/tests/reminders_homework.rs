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
        remaining_min: 120,
    };
    assert_eq!(homework_text(&r), "Essay 2 (due in 2 h, 2026-12-02 09:00)");
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

#[test]
fn a_reminder_states_the_time_left_not_the_lead_it_fired_at() {
    use studyfrog_calendar::homework_reminders::DueHomeworkReminder;
    use studyfrog_core::civil::LocalTime;
    use studyfrog_ui::reminder_ui::homework_text;
    let due = |remaining_min, lead_min| DueHomeworkReminder {
        item: Uid(8),
        title: "Kurzquiz".into(),
        due: LocalTime {
            date: date_new(2026, 11, 23).unwrap(),
            minute: 22 * 60 + 30,
        },
        lead_min,
        remaining_min,
    };
    // Due in 68 minutes, shown at the 2-hour lead: the time left is what the reader needs.
    assert_eq!(
        homework_text(&due(68, 120)),
        "Kurzquiz (due in 1 h 8 min, 2026-11-23 22:30)"
    );
    assert_eq!(
        homework_text(&due(15, 15)),
        "Kurzquiz (due in 15 min, 2026-11-23 22:30)"
    );
    assert_eq!(
        homework_text(&due(11 * 60, 24 * 60)),
        "Kurzquiz (due in 11 h, 2026-11-23 22:30)"
    );
    assert_eq!(
        homework_text(&due(1440, 1440)),
        "Kurzquiz (due in 1 day, 2026-11-23 22:30)"
    );
}
