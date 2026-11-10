#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Whole headless control sessions over in-memory streams, against a real vault and database.

mod common;
mod control_common;

use control_common::*;

const A: &str = control_common::CARD_A;

#[test]
fn a_session_navigates_queries_creates_and_edits_through_one_stream() {
    let (vault, logs) = (sample_vault("ctl-session"), temp_dir("ctl-session-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let input = format!(
        "state\n\n{}\nnav library\nquery type = card order by title\n\
         item create card \"Le chat\" --text cat --answer \"le chat\" --stack french\n\
         item edit {A} back la réponse\nitem color {A} \"#336699\"\nstack create Bio\n\
         item move {A} bio\nback\nbogus\n",
        json!({"id": "n1", "cmd": "nav", "args": ["calendar"]})
    );
    let r = session(&mut e, &logs.path, &input);
    assert_eq!(r.len(), 11, "one answer per non-blank line: {r:#?}");
    assert_eq!(r[0]["result"]["vault"]["stacks"][0]["title"], "French");
    assert_eq!(
        (&r[1]["id"], &r[1]["result"]["screen"]),
        (&json!("n1"), &json!("calendar"))
    );
    assert_eq!(r[2]["result"]["library"]["total"], 4);
    let rows = r[3]["result"]["rows"].as_array().unwrap();
    let titles: Vec<&str> = rows.iter().map(|x| x["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["A", "B"]);
    assert_eq!(r[4]["result"]["notice"]["text"], "Created \"Le chat\".");
    let card = std::fs::read_to_string(vault.path.join("bio/a.md")).unwrap();
    assert!(
        card.contains("la réponse") && card.contains("#336699"),
        "{card}"
    );
    assert_eq!(r[7]["ok"], true, "{}", r[7]);
    assert_eq!(r[8]["ok"], true, "{}", r[8]);
    assert!(
        vault.path.join("bio/a.md").exists(),
        "the card moved into the new stack"
    );
    assert_eq!(r[9]["ok"], true);
    assert!(
        r[10]["error"]
            .as_str()
            .unwrap()
            .contains("unknown command bogus")
    );
    let created = std::fs::read_dir(vault.path.join("french"))
        .unwrap()
        .count();
    assert_eq!(created, 4, "_stack.md, b.md, c.md and the new card");
}

#[test]
fn a_rehearsal_is_stepped_through_and_its_reviews_are_saved() {
    let (vault, logs) = (sample_vault("ctl-run"), temp_dir("ctl-run-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let input = "rehearse setup --preset spaced --stack french\nrehearse start\nrehearse mark easy\n\
                 rehearse flip\nrehearse rate good\nrehearse skip\nrehearse pause\nrehearse resume\n\
                 rehearse stop\nrehearse leave\n";
    let r = session(&mut e, &logs.path, input);
    assert!(r.iter().all(|x| x["ok"] == true), "{r:#?}");
    assert_eq!(
        r[0]["result"]["rehearse"]["preview"],
        json!({"status": "count", "items": 3})
    );
    let run = |i: usize| r[i]["result"]["rehearse"]["run"].clone();
    assert_eq!(
        (run(1)["state"].clone(), run(1)["phase"].clone()),
        (json!("running"), json!("shown"))
    );
    assert_eq!(
        run(1)["answer"],
        Value::Null,
        "the answer is hidden until flipped"
    );
    assert_eq!(run(3)["phase"], "revealed");
    assert!(run(3)["answer"].is_string());
    assert_eq!(run(4)["position"], 1);
    assert_eq!(run(6)["paused"], true);
    assert_eq!(run(7)["paused"], false);
    assert_eq!(run(8)["state"], "aborted");
    assert_eq!(run(9), Value::Null);
    let conn = &e.worker.session.as_ref().unwrap().conn;
    let reviews: i64 = conn
        .query_row("SELECT COUNT(*) FROM review", [], |x| x.get(0))
        .unwrap();
    assert_eq!(reviews, 1, "the one rating was saved");
}

#[test]
fn the_policy_guards_destructive_and_writing_commands() {
    let (vault, logs) = (sample_vault("ctl-policy"), temp_dir("ctl-policy-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let refused = ask(&mut e, &format!("item delete {A}"));
    assert!(refused["error"].as_str().unwrap().contains("destructive"));
    assert!(vault.path.join("french/a.md").exists());
    e.policy.allow_destructive = true;
    let deleted = ask(&mut e, &format!("item delete {A}"));
    assert_eq!(deleted["ok"], true, "{deleted}");
    assert!(!vault.path.join("french/a.md").exists());
    e.policy = ControlPolicy {
        allow_write: false,
        allow_destructive: false,
    };
    let r = ask(&mut e, "item create note N");
    assert!(r["error"].as_str().unwrap().contains("read-only"));
    assert_eq!(ask(&mut e, "query type = note")["result"]["count"], 2);
    assert!(
        ask(&mut e, "screenshot x.png")["error"]
            .as_str()
            .unwrap()
            .contains("read-only")
    );
    e.policy.allow_write = true;
    let shot = ask(&mut e, "screenshot x.png");
    assert!(shot["error"].as_str().unwrap().contains("headless"));
}

#[test]
fn exports_imports_and_failures_answer_with_their_outcome() {
    let (vault, logs) = (sample_vault("ctl-xfer"), temp_dir("ctl-xfer-logs"));
    let out = temp_dir("ctl-xfer-out");
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(
        &mut e,
        &format!("export csv {} --stack french", out.path.display()),
    );
    assert_eq!(r["ok"], true, "{r}");
    let csv = std::fs::read_dir(&out.path)
        .unwrap()
        .flatten()
        .next()
        .unwrap()
        .path();
    let r = ask(&mut e, &format!("import {} --stack french", csv.display()));
    assert_eq!(r["ok"], true, "{r}");
    assert!(
        r["result"]["notice"]["text"]
            .as_str()
            .unwrap()
            .starts_with("Imported")
    );
    let r = ask(&mut e, "import /no/such/file.csv");
    assert_eq!(r["ok"], false, "{r}");
    let missing = ask(&mut e, "item open 0000000000000000000000000Z");
    assert_eq!(missing["ok"], false, "{missing}");
    assert!(ask(&mut e, "stack settings french")["result"]["folder"] == "french");
}

#[test]
fn logs_are_read_by_level_and_failures_are_logged_as_control_entries() {
    let (vault, logs) = (sample_vault("ctl-logs"), temp_dir("ctl-logs-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let input = "item create note Hello --stack french\nitem create card Title --text Front --answer Back\nclose\nnav nowhere\nlogs --tail 10\nlogs --level warn\nlogs --tail 1\n";
    let r = session(&mut e, &logs.path, input);
    assert!(
        r[1]["error"].as_str().unwrap().contains("Choose the stack"),
        "{}",
        r[1]
    );
    assert_eq!(
        r[2]["result"]["modal"],
        Value::Null,
        "close closed the dialog"
    );
    let r = &r[4..];
    let entries = |i: usize| r[i]["result"]["entries"].as_array().unwrap().clone();
    let all = entries(0);
    assert!(
        all.iter()
            .any(|x| x["source"] == "notice" && x["message"] == "Created \"Hello\".")
    );
    assert!(
        all.iter()
            .any(|x| x["source"] == "control" && x["level"] == "warn")
    );
    assert!(entries(1).iter().all(|x| x["level"] != "info"));
    assert_eq!(entries(2).len(), 1);
    assert_eq!(r[0]["result"]["following"], false);
}
