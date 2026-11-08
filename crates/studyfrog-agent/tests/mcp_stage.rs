#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Staging tools under the policy, and staged proposals as JSON.

mod mcp_common;

use mcp_common::*;
use studyfrog_agent::mcp_stage::{
    StagedProposal, diary_change_of, parse_staged, session_change_of, staged_json,
};
use studyfrog_agent::wiki::Change;

fn with_stager() -> McpRegistry<Ctx> {
    let mut stage = tool("stage", WRITE, change);
    stage.staged = true;
    register_tool(registry(), stage)
}

fn ask(reg: &McpRegistry<Ctx>, policy: McpPolicy, c: &mut Ctx, msg: &Value) -> Value {
    serde_json::from_str(&handle_line(reg, &policy, c, &msg.to_string()).unwrap()).unwrap()
}

#[test]
fn a_staging_tool_is_listed_and_runs_without_permission_to_write() {
    let reg = with_stager();
    let mut c = Ctx::default();
    let list = ask(
        &reg,
        McpPolicy::default(),
        &mut c,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
    );
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"stage") && !names.contains(&"change") && !names.contains(&"wipe"));
    let call =
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "stage"}});
    let answer = ask(&reg, McpPolicy::default(), &mut c, &call);
    assert_eq!(answer["result"]["isError"], false);
    assert_eq!(c.changes, 1);
}

#[test]
fn diary_and_session_arguments_are_checked() {
    let ok = diary_change_of(
        &json!({"date": "2026-11-07", "ratings": {"focus": 3}, "study_minutes": 20}),
    );
    let Ok(Change::LogDiary(d)) = ok else {
        panic!("{ok:?}")
    };
    assert_eq!(
        (d.minutes, d.ratings),
        (Some(20), vec![("focus".to_owned(), 3)])
    );
    for bad in [
        json!({"date": "7.11.2026"}),
        json!({"date": "2026-11-07", "ratings": {"focus": 0}}),
        json!({"date": "2026-11-07", "ratings": {"bad name": 2}}),
        json!({"date": "2026-11-07", "study_minutes": 5000}),
    ] {
        assert!(diary_change_of(&bad).is_err(), "{bad}");
    }
    let stacks = vec!["french".to_owned()];
    let plan = json!({"title": "Drill", "date": "2026-11-09", "start": "07:05", "duration_minutes": 30, "stack": "french", "kind": "exam"});
    let Ok(Change::PlanSession(s)) = session_change_of(&plan, &stacks) else {
        panic!()
    };
    assert_eq!((s.start_minute, s.kind.as_str()), (7 * 60 + 5, "exam"));
    let mut wrong = plan.clone();
    wrong["stack"] = json!("latin");
    assert!(
        session_change_of(&wrong, &stacks)
            .unwrap_err()
            .contains("latin")
    );
    wrong["stack"] = json!("french");
    wrong["kind"] = json!("party");
    assert!(session_change_of(&wrong, &stacks).is_err());
}

#[test]
fn a_staged_proposal_reads_back_and_is_checked_again() {
    let stacks = vec!["french".to_owned()];
    let diary = diary_change_of(&json!({"date": "2026-11-07", "body": "ok"})).unwrap();
    let plan = session_change_of(
        &json!({"title": "Drill", "date": "2026-11-09", "start": "18:00", "duration_minutes": 30, "stack": "french"}),
        &stacks,
    )
    .unwrap();
    let card = Change::CreateCard {
        stack: "french".into(),
        front: "chat".into(),
        back: "cat".into(),
        source: None,
    };
    let p = StagedProposal {
        id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
        created_ms: 7,
        tool: "vault_propose_items".into(),
        summary: "Words".into(),
        changes: vec![card, diary, plan],
    };
    let text = staged_json(&p);
    assert_eq!(
        parse_staged(&text, &stacks).unwrap(),
        (p.clone(), Vec::new())
    );
    let (later, rejected) = parse_staged(&text, &[]).unwrap();
    assert_eq!(
        later.changes.len(),
        1,
        "only the diary entry needs no stack"
    );
    assert_eq!(
        rejected.len(),
        2,
        "the stack was removed since: {rejected:?}"
    );
}
