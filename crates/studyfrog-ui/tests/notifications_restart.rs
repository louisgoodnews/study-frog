#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Notifications are kept in the vault's database: they survive closing the app and reopening the vault.

mod common;

use common::{run_to_idle, sample_vault, temp_dir};
use studyfrog_ui::chrome::{ChromeMsg, unread};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::notify::NotifyMsg;

fn start(cfg: &std::path::Path) -> (Model, studyfrog_ui::exec::Worker) {
    let mut worker = worker_new(Some(cfg.to_path_buf()));
    let mut model = Model::default();
    run_to_idle(&mut model, &mut worker, vec![Effect::LoadAppSettings]);
    (model, worker)
}

fn raise(model: &mut Model, worker: &mut studyfrog_ui::exec::Worker, text: &str) {
    model.state.wall_ms = 1_800_000_000_000;
    apply(model, Msg::Chrome(ChromeMsg::SearchEdited("q".into())));
    let fx = apply(
        model,
        Msg::Chrome(ChromeMsg::SearchResults("q".into(), Err(text.into()))),
    );
    run_to_idle(model, worker, fx);
}

#[test]
fn notifications_and_their_read_state_survive_a_restart() {
    let vault = sample_vault("notify-restart");
    let cfg = temp_dir("cfg-notify");
    std::fs::write(
        cfg.path.join("app.toml"),
        format!("vault = {:?}\n", vault.path.display().to_string()),
    )
    .unwrap();
    {
        let (mut model, mut worker) = start(&cfg.path);
        model.state.wall_ms = 1_800_000_000_000;
        assert!(
            model.state.chrome.notify_loaded,
            "the stored list was read at start"
        );
        raise(&mut model, &mut worker, "first");
        raise(&mut model, &mut worker, "second");
        let id = model.state.chrome.notifications[0].id;
        let fx = apply(
            &mut model,
            Msg::Chrome(ChromeMsg::Notify(NotifyMsg::MarkRead(id, true))),
        );
        run_to_idle(&mut model, &mut worker, fx);
        let doomed = model.state.chrome.notifications[1].id;
        raise(&mut model, &mut worker, "third");
        let fx = apply(
            &mut model,
            Msg::Chrome(ChromeMsg::Notify(NotifyMsg::Delete(doomed))),
        );
        run_to_idle(&mut model, &mut worker, fx);
    }
    let (mut model, _worker) = start(&cfg.path);
    model.state.wall_ms = 1_800_000_000_000 + 1000;
    let list = &model.state.chrome.notifications;
    let texts: Vec<&str> = list.iter().map(|n| n.text.as_str()).collect();
    assert_eq!(
        texts,
        ["Search failed: first", "Search failed: third"],
        "the deleted one stays gone"
    );
    assert_eq!(
        list.iter().map(|n| n.read).collect::<Vec<_>>(),
        [true, false]
    );
    assert_eq!(unread(&model.state.chrome), 1);
}
