#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0234: exam runs are stored with the mode `exam`, old runs survive the upgrade, results round-trip.

use studyfrog_core::exam::{ExamGrade, ExamResult};
use studyfrog_core::plan::plan_sequential;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::db::open_memory;
use studyfrog_store::exam_results::{load_exam_result, recent_exam_results, save_exam_result};
use studyfrog_store::migrate::{migrate, schema_version};
use studyfrog_store::run_load::load_run;
use studyfrog_store::runs::insert_run;
use studyfrog_store::schema::MIGRATIONS;

fn run(uid: u128, mode: RunMode) -> Run {
    Run {
        uid: Uid(uid),
        mode,
        plan: plan_sequential(),
        queue: vec![Uid(10), Uid(11)],
        state: RunState::Finished,
        started_at: None,
    }
}

fn result(run: u128, percent: f32, grade: ExamGrade, at: i64) -> ExamResult {
    ExamResult {
        run: Uid(run),
        items: 2,
        answered: 2,
        percent,
        grade,
        passed: grade != ExamGrade::F,
        engine: "words".into(),
        graded_at_ms: at,
    }
}

#[test]
fn runs_from_before_the_upgrade_are_kept_and_exam_runs_can_be_stored() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, &MIGRATIONS[..24]).unwrap();
    insert_run(&conn, &run(1, RunMode::Answer)).unwrap();
    let refused = insert_run(&conn, &run(2, RunMode::Exam));
    assert!(refused.is_err(), "version 24 does not know exam runs");
    migrate(&mut conn, MIGRATIONS).unwrap();
    assert_eq!(schema_version(&conn).unwrap(), 26);
    assert_eq!(
        load_run(&conn, Uid(1)).unwrap().unwrap(),
        run(1, RunMode::Answer)
    );
    insert_run(&conn, &run(2, RunMode::Exam)).unwrap();
    assert_eq!(
        load_run(&conn, Uid(2)).unwrap().unwrap().mode,
        RunMode::Exam
    );
}

#[test]
fn exam_results_round_trip_and_list_newest_first() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    assert_eq!(load_exam_result(&conn, Uid(5)).unwrap(), None);
    let a = result(5, 82.5, ExamGrade::BPlus, 100);
    let b = result(6, 40.0, ExamGrade::F, 200);
    save_exam_result(&conn, &a).unwrap();
    save_exam_result(&conn, &b).unwrap();
    assert_eq!(load_exam_result(&conn, Uid(5)).unwrap(), Some(a.clone()));
    assert_eq!(recent_exam_results(&conn, 10).unwrap(), [b, a.clone()]);
    let regraded = result(5, 95.0, ExamGrade::APlus, 300);
    save_exam_result(&conn, &regraded).unwrap();
    assert_eq!(load_exam_result(&conn, Uid(5)).unwrap(), Some(regraded));
}

#[test]
fn a_broken_grade_is_refused_by_the_table() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    let bad = conn.execute(
        "INSERT INTO exam_result VALUES (?1, 1, 1, 50.0, 'E', 0, '', 1)",
        [format_ulid(Uid(9))],
    );
    assert!(bad.is_err());
}
