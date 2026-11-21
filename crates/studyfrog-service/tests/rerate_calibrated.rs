#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Rerating a first answer to the same rating keeps its schedule, even when the starting difficulty
//! was calibrated (STORY-0177): the replay uses the calibration in force when the review was made.

use std::fs;

use studyfrog_core::event::{EventKind as RunEventKind, RunEvent};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating::Good;
use studyfrog_core::run::RunState;
use studyfrog_core::settings::vault_settings_default;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_service::rehearse::{RatingTiming, StepContext, record_step};
use studyfrog_service::rerate::rerate_latest;
use studyfrog_service::session::open_vault;
use studyfrog_srs::fsrs::initial_state;
use studyfrog_srs::params::params_default;
use studyfrog_store::schedule::load_mem_state;

const DAY: i64 = 86_400_000;
const START: i64 = 1_700_000_000_000;

fn vault(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-rerate-cal-{label}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        format!(
            "---\nid: {}\nkind: stack\ntitle: S\n---\n",
            format_ulid(Uid(1))
        ),
    )
    .unwrap();
    for (n, file) in [(0xA01u128, "a.md"), (0xB01, "b.md")] {
        fs::write(
            dir.join("s").join(file),
            format!(
                "---\nid: {}\nkind: card\ntitle: {file}\n---\nq\n???\na\n",
                format_ulid(Uid(n))
            ),
        )
        .unwrap();
    }
    dir
}

fn rate(conn: &mut rusqlite::Connection, item: Uid, at: i64, answer_ms: u32, expected_ms: u32) {
    let settings = vault_settings_default();
    let out = StepOutput {
        events: vec![RunEvent {
            run: item,
            seq: 0,
            t_ms: 0,
            item: Some(item),
            kind: RunEventKind::ItemRated(Good),
        }],
        latency: None,
    };
    let ctx = StepContext {
        run: item,
        effect: ScheduleEffect::UpdateSchedule,
        state: RunState::Running,
        timing: Some(RatingTiming {
            answer_ms,
            expected_ms,
        }),
    };
    record_step(conn, &ctx, &out, UnixMs(at), &settings).unwrap();
}

#[test]
fn rerating_a_calibrated_first_answer_to_the_same_rating_keeps_the_schedule() {
    let dir = vault("keep");
    let (mut session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let (a, b) = (Uid(0xA01), Uid(0xB01));
    // A's first answer is slow; B's first answer comes later, so its start is calibrated from A.
    rate(&mut session.conn, a, START, 9_000, 4_000);
    rate(&mut session.conn, b, START + DAY, 2_000, 4_000);
    let before = load_mem_state(&session.conn, b).unwrap().unwrap();
    let plain = initial_state(Good, &params_default());
    assert_ne!(
        before.state.difficulty, plain.difficulty,
        "the start was calibrated"
    );

    rerate_latest(&mut session, b, Good, Good).unwrap();
    let after = load_mem_state(&session.conn, b).unwrap().unwrap();
    assert_eq!(after.state, before.state);
    let _ = fs::remove_dir_all(&dir);
}
