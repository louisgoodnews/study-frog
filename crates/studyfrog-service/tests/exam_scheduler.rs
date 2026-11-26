#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Service integration tests for the Exam Scheduler (EPIC-076, TASK-0311).

use std::fs;
use std::path::Path;

use studyfrog_calendar::exam_plan::{ExamPlan, ExamTarget, PrepWindow};
use studyfrog_core::civil::Date;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::exam_scheduler::{
    delete_vault_exam_plan, estimate_exam_window, list_vault_exam_plans,
    rebalance_and_save_exam_plan, save_and_apply_exam_plan,
};
use studyfrog_service::session::open_vault;

fn write_file(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[test]
fn exam_scheduler_proposes_window_applies_persists_and_rebalances() {
    let dir = std::env::temp_dir().join(format!(
        "sf_svc_exam_sched_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    write_file(
        &dir,
        "bio/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Biology\n---\n",
    );
    for i in 1..=6u8 {
        write_file(
            &dir,
            &format!("bio/c{i}.md"),
            &format!(
                "---\nid: 0000000000000000000000001{i}\nkind: card\ntitle: Card {i}\n---\nFront {i}\n???\nBack {i}\n"
            ),
        );
    }

    let (mut session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let today = Date {
        year: 2026,
        month: 11,
        day: 1,
    };
    let exam_date = Date {
        year: 2026,
        month: 11,
        day: 25,
    };
    let now = UnixMs(1_795_000_000_000);
    let targets = vec![ExamTarget::Stack("bio".into())];

    let proposed = estimate_exam_window(&session, &targets, today, exam_date, 20, now).unwrap();
    assert!(proposed.weeks >= 1);
    assert!(!proposed.reasons.is_empty());

    let plan = ExamPlan {
        id: Uid(500),
        title: "Biology Midterm".into(),
        exam_date,
        window: PrepWindow::Proposed {
            weeks: proposed.weeks,
            reasons: proposed.reasons.clone(),
        },
        targets,
        blocks: Vec::new(),
        created_ms: 0,
        updated_ms: 0,
    };

    let (saved, out) = save_and_apply_exam_plan(&mut session, &plan, today, 20, now).unwrap();
    assert!(!saved.blocks.is_empty());
    assert!(!out.blocks.is_empty());

    let listed = list_vault_exam_plans(&session).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title, "Biology Midterm");

    // Rebalance later in the window
    let later = Date {
        year: 2026,
        month: 11,
        day: 15,
    };
    let (rebalanced, _) =
        rebalance_and_save_exam_plan(&mut session, Uid(500), later, 20, now).unwrap();
    assert!(!rebalanced.blocks.is_empty());

    assert!(delete_vault_exam_plan(&session, Uid(500)).unwrap());
    assert!(list_vault_exam_plans(&session).unwrap().is_empty());

    let _ = fs::remove_dir_all(&dir);
}
