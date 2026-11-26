#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! SQLite round-trip tests for ExamPlan persistence (EPIC-076, TASK-0306).

use studyfrog_calendar::exam_plan::{ExamPlan, ExamScheduleBlock, ExamTarget, PrepWindow};
use studyfrog_core::civil::Date;
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_db;
use studyfrog_store::exam_plan::{
    delete_exam_plan, list_exam_plans, load_exam_plan, save_exam_plan,
};
use studyfrog_store::migrate::migrate;
use studyfrog_store::schema::MIGRATIONS;

#[test]
fn exam_plan_round_trip_update_and_cascade_delete() {
    let dir = std::env::temp_dir().join(format!(
        "sf_store_exam_plan_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("index.sqlite");

    {
        let mut conn = open_db(&db_path).unwrap();
        migrate(&mut conn, MIGRATIONS).unwrap();

        let plan = ExamPlan {
            id: Uid(101),
            title: "Diagnostik Klausur".into(),
            exam_date: Date {
                year: 2026,
                month: 12,
                day: 20,
            },
            window: PrepWindow::Proposed {
                weeks: 3,
                reasons: vec!["97 linked items".into()],
            },
            targets: vec![ExamTarget::Stack("psych".into()), ExamTarget::Item(Uid(42))],
            blocks: vec![ExamScheduleBlock {
                date: Date {
                    year: 2026,
                    month: 12,
                    day: 1,
                },
                item_uids: vec![Uid(42), Uid(43)],
                new_count: 1,
                review_count: 1,
            }],
            created_ms: 1000,
            updated_ms: 2000,
        };
        save_exam_plan(&mut conn, &plan).unwrap();

        let loaded = load_exam_plan(&conn, Uid(101)).unwrap().unwrap();
        assert_eq!(loaded, plan);

        let updated = ExamPlan {
            title: "Diagnostik Final".into(),
            window: PrepWindow::UserWeeks(4),
            updated_ms: 3000,
            ..plan
        };
        save_exam_plan(&mut conn, &updated).unwrap();
        let all = list_exam_plans(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Diagnostik Final");
        assert_eq!(all[0].window, PrepWindow::UserWeeks(4));

        assert!(delete_exam_plan(&conn, Uid(101)).unwrap());
        assert!(load_exam_plan(&conn, Uid(101)).unwrap().is_none());
    }

    let _ = std::fs::remove_dir_all(&dir);
}
