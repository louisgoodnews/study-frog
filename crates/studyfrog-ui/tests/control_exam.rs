#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0242: an exam set up, started, answered, submitted, graded and finished over the control protocol.

mod common;
mod control_common;

use control_common::*;

fn refusals(r: &[Value]) {
    assert!(
        r[2]["error"]
            .as_str()
            .unwrap()
            .contains("only written answers")
    );
    assert!(
        r[6]["error"]
            .as_str()
            .unwrap()
            .contains("the answer is empty")
    );
    assert!(r[7]["error"].as_str().unwrap().contains("write an answer"));
    assert!(r[8]["error"].as_str().unwrap().contains("not over"));
}

fn stored(e: &Engine) {
    let conn = &e.worker.session.as_ref().unwrap().conn;
    let stored: f64 = conn
        .query_row("SELECT percent FROM exam_result", [], |x| x.get(0))
        .unwrap();
    assert_eq!(stored, 50.0);
    let reviews: i64 = conn
        .query_row("SELECT COUNT(*) FROM review", [], |x| x.get(0))
        .unwrap();
    assert_eq!(reviews, 0, "an exam never changes the schedule");
}

#[test]
fn an_agent_takes_a_whole_exam_and_reads_its_grade() {
    let (vault, logs) = (sample_vault("ctl-exam"), temp_dir("ctl-exam-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let input = "rehearse setup --preset all --mode exam\nrehearse start\nrehearse flip\nrehearse skip\n\
                 rehearse rate good\nrehearse mark easy\nrehearse submit\nrehearse answer   \nrehearse finish\n\
                 rehearse type a\nrehearse submit\nrehearse answer no idea at all\nrehearse finish\nstate\n";
    let r = session(&mut e, &logs.path, input);
    assert_eq!(r.len(), 14, "{r:#?}");
    let ok: Vec<bool> = r.iter().map(|x| x["ok"] == true).collect();
    let want = [
        true, true, false, false, false, false, false, false, false, true, true, true, true, true,
    ];
    assert_eq!(ok, want, "{r:#?}");
    assert_eq!(r[0]["result"]["rehearse"]["mode"], "exam");
    assert_eq!(
        r[0]["result"]["rehearse"]["preview"],
        json!({"status": "count", "items": 2}),
        "notes are left out"
    );
    refusals(&r);
    let run = |i: usize| r[i]["result"]["rehearse"]["run"].clone();
    assert_eq!(run(9)["typed"], "a");
    let exam = run(10)["exam"].clone();
    assert_eq!(
        (exam["answered"].clone(), exam["items"].clone()),
        (json!(1), json!(2))
    );
    assert_eq!(exam["locked"][0]["answer"], "a");
    assert_eq!(
        run(10)["position"],
        1,
        "the locked answer moved the exam on"
    );
    let done = run(11)["exam"].clone();
    assert_eq!(done["graded"], true, "{done:#}");
    assert_eq!(
        done["result"],
        json!({"percent": 50.0, "grade": "F", "german": "5.0", "passed": false,
        "items": 2, "answered": 2, "engine": done["result"]["engine"].clone()})
    );
    assert_eq!(
        r[13]["result"]["rehearse"]["run"],
        Value::Null,
        "finish left the result"
    );
    stored(&e);
}

#[test]
fn a_stopped_exam_is_not_graded_and_finish_needs_an_exam() {
    let (vault, logs) = (
        sample_vault("ctl-exam-stop"),
        temp_dir("ctl-exam-stop-logs"),
    );
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let first = ask(&mut e, "rehearse finish");
    assert!(first["error"].as_str().unwrap().contains("no exam"));
    let input = "rehearse setup --preset all --mode exam\nrehearse start\nrehearse answer a\nrehearse stop\n";
    let r = session(&mut e, &logs.path, input);
    assert!(r.iter().all(|x| x["ok"] == true), "{r:#?}");
    let exam = r[3]["result"]["rehearse"]["run"]["exam"].clone();
    assert_eq!(
        (exam["stopped"].clone(), exam["graded"].clone()),
        (json!(true), json!(false))
    );
    let r = session(&mut e, &logs.path, "rehearse finish\n");
    assert_eq!(r[0]["ok"], true);
    let conn = &e.worker.session.as_ref().unwrap().conn;
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM exam_result", [], |x| x.get(0))
        .unwrap();
    assert_eq!(n, 0);
}
