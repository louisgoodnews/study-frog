#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for TASK-0173: Post-rehearsal diary prompt on the run summary screen.

mod common;

use common::sample_vault;
use common::views::{harness, running_model};
use eframe::egui::Key;
use egui_kittest::kittest::Queryable;
use studyfrog_core::diary::RatingValue;
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_md::ulid::format_ulid;
use studyfrog_ui::diary_types::DiaryEffect;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};

fn finish_run(h: &mut egui_kittest::Harness<'_, common::views::App>) {
    h.run_steps(2);
    h.key_press(Key::Space); // flip item 1
    h.run_steps(2);
    h.key_press(Key::Num3); // rate item 1 Good
    h.run_steps(2);
    h.key_press(Key::S); // skip item 2 -> run finishes
    h.run_steps(2);
    let state = h
        .state()
        .0
        .state
        .rehearse
        .run
        .as_ref()
        .unwrap()
        .session
        .run
        .state;
    assert_eq!(state, RunState::Finished);
}

#[test]
fn prompt_appears_on_run_summary() {
    let mut h = harness(running_model(RunMode::Flip));
    finish_run(&mut h);

    assert!(h.query_by_label("Run finished").is_some());
    assert!(
        h.query_by_label_contains("Log your session in the diary?")
            .is_some()
    );
    assert!(h.query_by_label_contains("Duration:").is_some());
    let run_uid = h
        .state()
        .0
        .state
        .rehearse
        .run
        .as_ref()
        .unwrap()
        .session
        .run
        .uid;
    assert!(h.query_by_label_contains(&format_ulid(run_uid)).is_some());
    assert!(h.query_by_label_contains("items reviewed").is_some());

    assert!(h.query_by_label("Focus & Attention:").is_some());
    assert!(h.query_by_label("Energy level:").is_some());
    assert!(h.query_by_label("Satisfaction & Retention:").is_some());

    assert!(h.query_by_label("Rate Energy level level 4").is_some());
    assert!(h.query_by_label("Reflection:").is_some());
    assert!(h.query_by_label("Save entry (Enter)").is_some());
    assert!(h.query_by_label("More details...").is_some());
    assert!(h.query_by_label("Skip (Esc)").is_some());
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn submitting_creates_diary_entry_linking_run_and_items() {
    let mut h = harness(running_model(RunMode::Flip));
    finish_run(&mut h);

    let run_uid = h
        .state()
        .0
        .state
        .rehearse
        .run
        .as_ref()
        .unwrap()
        .session
        .run
        .uid;
    let reviewed = h
        .state()
        .0
        .state
        .rehearse
        .run
        .as_ref()
        .unwrap()
        .session
        .queue
        .clone();

    h.get_by_label("Rate Energy level level 4").click();
    h.run_steps(2);
    h.get_by_label("Rate Focus & Attention level 5").click();
    h.run_steps(2);
    h.get_by_label("Rate Satisfaction & Retention level 4")
        .click();
    h.run_steps(2);

    h.get_by_label("Reflection").focus();
    h.get_by_label("Reflection").type_text("Felt in the zone");
    h.run_steps(2);

    h.get_by_label("Save entry (Enter)").click();
    h.run_steps(2);

    let save_effect = h
        .state()
        .1
        .iter()
        .find_map(|e| {
            if let Effect::Diary(DiaryEffect::SaveEntry(draft)) = e {
                Some(draft.clone())
            } else {
                None
            }
        })
        .expect("SaveEntry effect was dispatched");

    assert_eq!(save_effect.runs, vec![run_uid]);
    assert_eq!(save_effect.items, reviewed);
    assert_eq!(
        save_effect.ratings.get("energy"),
        Some(&RatingValue::Int(4))
    );
    assert_eq!(save_effect.ratings.get("focus"), Some(&RatingValue::Int(5)));
    assert_eq!(
        save_effect.ratings.get("satisfaction"),
        Some(&RatingValue::Int(4))
    );
    assert_eq!(save_effect.content, "Felt in the zone");
    assert!(save_effect.study_minutes.is_some());

    let dir = sample_vault("diary-prompt");
    let mut worker = worker_new(None);
    let msgs = execute(
        &mut worker,
        Effect::OpenVault(dir.path.to_string_lossy().into()),
        1000,
    );
    assert!(
        msgs.iter()
            .any(|m| matches!(m, studyfrog_ui::msg::Msg::VaultOpened(Ok(_))))
    );

    let save_msgs = execute(
        &mut worker,
        Effect::Diary(DiaryEffect::SaveEntry(save_effect.clone())),
        2000,
    );
    assert!(save_msgs.iter().any(|m| {
        matches!(
            m,
            studyfrog_ui::msg::Msg::Diary(studyfrog_ui::diary_msg::DiaryMsg::Saved(Ok(_)))
        )
    }));

    let date_str = studyfrog_core::civil::format_date(save_effect.date);
    let md_path = dir.path.join(format!("diary/{date_str}.md"));
    assert!(md_path.exists(), "Diary file was written");
    let content = std::fs::read_to_string(md_path).unwrap();
    assert!(content.contains(&format_ulid(run_uid)));
    for item_uid in &reviewed {
        assert!(content.contains(&format_ulid(*item_uid)));
    }
    assert!(content.contains("energy: 4"));
    assert!(content.contains("focus: 5"));
    assert!(content.contains("satisfaction: 4"));
    assert!(content.contains("Felt in the zone"));

    assert!(h.query_by_label("Session logged in diary.").is_some());
    h.get_by_label("Back to setup").click();
    h.run_steps(2);
    assert!(h.state().0.state.rehearse.run.is_none());
}
