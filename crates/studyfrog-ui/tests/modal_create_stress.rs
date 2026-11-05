#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Empirical stress tests for Milestone 4: Create Menu Due Date/Priority & Note Modal Expansion.

mod common;

use common::flow::{opened, send};
use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::Date;
use studyfrog_core::priority::{PRIORITIES, priority_name};
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::create_more::NewKind;
use studyfrog_store::overview::load_indexed_items;
use studyfrog_ui::exec::Worker;
use studyfrog_ui::modal::{Field, Modal, ModalMsg, problem};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::view::modal_size::{NORMAL, reset, set_width, widest, width};

fn dialog_model(kind: Option<NewKind>) -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
    if let Some(k) = kind {
        apply(&mut m, Msg::Modal(ModalMsg::SetKind(k)));
    }
    m
}

fn send_modal(model: &mut Model, worker: &mut Worker, msg: ModalMsg) {
    send(model, worker, Msg::Modal(msg));
}

#[test]
fn test_stack_priority_all_levels_ui_and_persistence() {
    let (vault, mut model, mut worker) = opened("m4-priority-stress");

    for p in PRIORITIES {
        let name = priority_name(p);
        let title = format!("Stack-{name}");
        send_modal(&mut model, &mut worker, ModalMsg::OpenCreate);
        send_modal(&mut model, &mut worker, ModalMsg::SetKind(NewKind::Stack));
        send_modal(&mut model, &mut worker, ModalMsg::SetParent(None));
        send_modal(
            &mut model,
            &mut worker,
            ModalMsg::Edit(Field::Title, title.clone()),
        );
        send_modal(&mut model, &mut worker, ModalMsg::SetPriority(p));
        send_modal(&mut model, &mut worker, ModalMsg::Submit);

        let slug = format!("stack-{name}");
        let stack_file = vault.path.join(&slug).join("_stack.md");
        let text = std::fs::read_to_string(&stack_file).unwrap();
        assert!(text.contains(&format!("priority: {name}")));

        let card_title = format!("CardIn-{name}");
        for msg in [
            ModalMsg::OpenCreate,
            ModalMsg::SetKind(NewKind::Card),
            ModalMsg::SetParent(Some(slug)),
            ModalMsg::Edit(Field::Title, card_title.clone()),
            ModalMsg::Edit(Field::Text, "Front".into()),
            ModalMsg::Edit(Field::Answer, "Back".into()),
            ModalMsg::SetPriority(p),
            ModalMsg::Submit,
        ] {
            send_modal(&mut model, &mut worker, msg);
        }

        let conn = &worker.session.as_ref().unwrap().conn;
        let items = load_indexed_items(conn).unwrap();
        let card = items
            .iter()
            .find(|i| {
                let id_str = format_ulid(i.uid);
                let title_in_db: String = conn
                    .query_row("SELECT title FROM item WHERE uid = ?1", [&id_str], |row| {
                        row.get(0)
                    })
                    .unwrap_or_default();
                title_in_db == card_title
            })
            .unwrap();
        assert_eq!(card.priority, p);
    }
}

#[test]
fn test_due_date_and_reminders_across_all_kinds_persistence() {
    let (_vault, mut model, mut worker) = opened("m4-due-stress");
    let kinds = [
        (NewKind::Stack, "Due-Stack", None, 20),
        (NewKind::Card, "Due-Card", Some("french"), 21),
        (NewKind::Note, "Due-Note", Some("french"), 22),
        (NewKind::Question, "Due-Question", Some("french"), 23),
    ];

    for (kind, title, parent, day) in kinds {
        send_modal(&mut model, &mut worker, ModalMsg::OpenCreate);
        send_modal(&mut model, &mut worker, ModalMsg::SetKind(kind));
        send_modal(
            &mut model,
            &mut worker,
            ModalMsg::SetParent(parent.map(str::to_owned)),
        );
        send_modal(
            &mut model,
            &mut worker,
            ModalMsg::Edit(Field::Title, title.into()),
        );
        send_modal(
            &mut model,
            &mut worker,
            ModalMsg::Edit(Field::Text, "Prompt / Front / Body".into()),
        );
        send_modal(
            &mut model,
            &mut worker,
            ModalMsg::Edit(Field::Answer, "Ans / Back".into()),
        );
        send_modal(
            &mut model,
            &mut worker,
            ModalMsg::Edit(Field::Due, format!("2026-12-{day:02}")),
        );
        for min in [15, 60, 60] {
            send_modal(&mut model, &mut worker, ModalMsg::ToggleReminder(min));
        }
        send_modal(&mut model, &mut worker, ModalMsg::Submit);

        let expected_title = format!("Due: {title}");
        let session = model
            .state
            .planner
            .sessions
            .iter()
            .find(|s| s.title == expected_title)
            .unwrap();
        assert_eq!(session.kind, EventKind::Deadline);
        assert_eq!(
            session.start.date,
            Date {
                year: 2026,
                month: 12,
                day
            }
        );
        assert_eq!(session.reminders, vec![15, 1440]);
        assert!(session.stack.is_some());
    }
}

#[test]
fn test_form_validation_and_adversarial_date_syntax() {
    let mut m = dialog_model(Some(NewKind::Card));
    for msg in [
        ModalMsg::SetParent(Some("french".into())),
        ModalMsg::Edit(Field::Title, "Card".into()),
        ModalMsg::Edit(Field::Text, "Front".into()),
        ModalMsg::Edit(Field::Answer, "Back".into()),
    ] {
        apply(&mut m, Msg::Modal(msg));
    }

    for valid in ["2026-12-31", "2026-01-01", "", "   "] {
        apply(&mut m, Msg::Modal(ModalMsg::Edit(Field::Due, valid.into())));
        let Some(Modal::Create(f)) = &m.state.modal else {
            panic!()
        };
        assert_eq!(problem(f), None);
    }

    let bad_dates = [
        "2026-02-31",
        "2026-13-40",
        "hello",
        "2026/12/31",
        "2026-1-1",
    ];
    for kind in [
        NewKind::Card,
        NewKind::Note,
        NewKind::Question,
        NewKind::Stack,
    ] {
        apply(&mut m, Msg::Modal(ModalMsg::SetKind(kind)));
        for bad in bad_dates {
            apply(&mut m, Msg::Modal(ModalMsg::Edit(Field::Due, bad.into())));
            let Some(Modal::Create(f)) = &m.state.modal else {
                panic!()
            };
            assert_eq!(problem(f), Some("The due date must look like 2026-12-31."));
        }
    }
}

#[test]
fn test_note_modal_auto_expansion_write_view_and_resizing() {
    let mut h = harness(dialog_model(Some(NewKind::Card)));
    h.run_steps(4);
    assert_eq!(width(&h.ctx, NewKind::Card), NORMAL);

    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::SetKind(NewKind::Note)),
    );
    h.run_steps(4);
    let full = widest(&h.ctx);
    assert_eq!(width(&h.ctx, NewKind::Note), full);
    assert!(h.query_by_label("Body: write").is_some() && h.query_by_label("Body: split").is_some());

    h.get_by_label("Shrink the dialog").click();
    h.run_steps(4);
    assert_eq!(width(&h.ctx, NewKind::Note), NORMAL);

    h.get_by_label("Enlarge the dialog").click();
    h.run_steps(4);
    assert_eq!(width(&h.ctx, NewKind::Note), full);

    set_width(&h.ctx, NewKind::Note, 750.0);
    assert_eq!(width(&h.ctx, NewKind::Note), 750.0);

    for (k, expected) in [(NewKind::Card, NORMAL), (NewKind::Note, 750.0)] {
        apply(&mut h.state_mut().0, Msg::Modal(ModalMsg::SetKind(k)));
        h.run_steps(4);
        assert_eq!(width(&h.ctx, k), expected);
    }

    reset(&h.ctx);
    assert_eq!(width(&h.ctx, NewKind::Note), full);
}
