#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0199: the notice after Create opens the new item, and so does its entry in the notification list.

mod common;

use common::views::open_model;
use studyfrog_core::uid::Uid;
use studyfrog_service::create_more::Created;
use studyfrog_ui::chrome::ChromeMsg;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::{CreateOutcome, Field, ModalMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::notify::{NotifyMsg, NotifyTarget};

const UID: Uid = Uid(0x0123_4567_89ab_cdef);

fn modal(m: &mut Model, msg: ModalMsg) -> Vec<Effect> {
    apply(m, Msg::Modal(msg))
}

fn create(m: &mut Model, another: bool, title: &str) {
    modal(m, ModalMsg::OpenCreate);
    modal(m, ModalMsg::Edit(Field::Text, "Q?".into()));
    modal(m, ModalMsg::Edit(Field::Answer, "A".into()));
    modal(m, ModalMsg::SetAnother(another));
    modal(m, ModalMsg::Submit);
    let outcome = Ok(Box::new(CreateOutcome {
        info: m.state.vault.clone().unwrap(),
        created: Created {
            title: title.into(),
            path: "french/q.md".into(),
            uid: Some(UID),
        },
        warnings: Vec::new(),
    }));
    modal(m, ModalMsg::Created(outcome));
}

fn loaded() -> Model {
    let mut m = open_model();
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::NotificationsLoaded(Ok(Vec::new()))),
    );
    m
}

#[test]
fn the_created_notice_links_to_the_new_item_and_the_list_keeps_the_link() {
    let mut m = loaded();
    create(&mut m, false, "Q?");
    let text = m.state.notice.clone().unwrap().text;
    assert_eq!(m.state.notice_link, Some((text.clone(), UID)));
    let n = m.state.chrome.notifications.last().unwrap().clone();
    assert_eq!(n.target, Some(NotifyTarget::Item(UID)));
    let fx = apply(
        &mut m,
        Msg::Chrome(ChromeMsg::Notify(NotifyMsg::Open(n.id))),
    );
    assert!(fx.contains(&Effect::LoadDocument(UID)));
}

#[test]
fn create_another_raises_a_linked_notice_for_each_item() {
    let mut m = loaded();
    create(&mut m, true, "One");
    create(&mut m, true, "Two");
    let targets: Vec<_> = m
        .state
        .chrome
        .notifications
        .iter()
        .map(|n| n.target)
        .collect();
    assert_eq!(targets, vec![Some(NotifyTarget::Item(UID)); 2]);
    assert!(m.state.modal.is_some(), "the dialog stays open");
}
