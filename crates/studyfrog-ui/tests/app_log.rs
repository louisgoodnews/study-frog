#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A notice raised in the window ends up in the application log files.

mod common;

use std::fs;

use common::views::open_model;
use studyfrog_ui::chrome::ChromeMsg;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::notify::{NotifyEffect, log_new};

const NOW: i64 = 1_800_000_000_000;

#[test]
fn a_notice_is_written_to_both_log_files() {
    let dir = std::env::temp_dir().join(format!("studyfrog-app-log-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);

    let mut model = open_model();
    model.state.wall_ms = NOW;
    let before = model.state.notice.clone();
    apply(&mut model, Msg::Chrome(ChromeMsg::SearchEdited("q".into())));
    apply(
        &mut model,
        Msg::Chrome(ChromeMsg::SearchResults(
            "q".into(),
            Err("the index is gone".into()),
        )),
    );
    let effect = log_new(&before, &model.state).expect("a new notice is logged");
    assert!(
        log_new(&model.state.notice, &model.state).is_none(),
        "the same notice is not logged twice"
    );
    let logs = vec![effect];

    let mut worker = worker_new(None);
    worker.state_dir = Some(dir.clone());
    for effect in logs {
        assert!(execute(&mut worker, effect, NOW * 1_000_000).is_empty());
    }

    let text = fs::read_to_string(dir.join("studyfrog.log")).unwrap();
    assert!(
        text.contains("ERROR notice: Search failed: the index is gone"),
        "{text}"
    );
    let json = fs::read_to_string(dir.join("console.jsonl")).unwrap();
    assert_eq!(json.lines().count(), 1, "{json}");
    assert!(
        json.contains(r#""message":"Search failed: the index is gone""#),
        "{json}"
    );
    assert!(json.contains(r#""level":"error""#), "{json}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_worker_without_a_log_directory_ignores_notices() {
    let mut worker = worker_new(None);
    let effect = Effect::Notifications(NotifyEffect::Log(
        studyfrog_ui::state::NoticeKind::Info,
        "nothing to write to".into(),
        NOW,
    ));
    assert!(execute(&mut worker, effect, NOW * 1_000_000).is_empty());
}
