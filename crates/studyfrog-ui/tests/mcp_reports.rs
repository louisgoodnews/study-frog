#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The MCP server's reports, diary, due queue, PDF pages and item schedules on a real vault, and that
//! none of them changes a file or a row of the index.

mod common;
mod mcp_common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use mcp_common::*;

fn with_schedule(c: &VaultCtx) {
    c.session
        .conn
        .execute(
            "INSERT INTO mem_state (item_uid, stack_uid, stability, difficulty, due_ms, last_review_ms) VALUES (?1, ?2, 3.5, 6.0, 1700000000000, 1690000000000)",
            [format_ulid(Uid(10)), format_ulid(Uid(1))],
        )
        .unwrap();
}

#[test]
fn every_report_kind_is_a_resource_and_unknown_kinds_are_refused() {
    let vault = sample_vault("mcp-reports");
    let mut c = ctx(&vault.path);
    for kind in [
        "retention",
        "forecast",
        "familiarity",
        "calibration",
        "runs",
        "latency",
        "leeches",
    ] {
        let answer = read(&mut c, &format!("studyfrog://reports/{kind}"));
        assert_eq!(
            answer["result"]["contents"][0]["mimeType"], "application/json",
            "{kind}: {answer}"
        );
        assert!(content(&answer, 0).is_object(), "{kind}");
    }
    let forecast = content(&read(&mut c, "studyfrog://reports/forecast"), 0);
    assert_eq!(forecast["reviews_per_day"].as_array().unwrap().len(), 30);
    assert!(forecast["backtest"]["simulated_history"].as_bool().unwrap());
    let unknown = read(&mut c, "studyfrog://reports/nonsense");
    assert_eq!(unknown["error"]["code"], -32002);
}

#[test]
fn the_analytics_and_forecast_tools_answer() {
    let vault = sample_vault("mcp-analytics");
    let mut c = ctx(&vault.path);
    let p = McpPolicy::default();
    let report = call(
        &mut c,
        p,
        "analytics_get_report",
        json!({"kind": "calibration"}),
    );
    assert_eq!(report["kind"], "calibration");
    assert_eq!(report["report"]["bias"], "WellCalibrated");
    let leeches = call(
        &mut c,
        p,
        "analytics_get_report",
        json!({"kind": "leeches"}),
    );
    assert!(
        leeches["report"]["leeches"].as_array().unwrap().is_empty(),
        "no history, no leeches"
    );
    let week = call(&mut c, p, "forecast_simulate_workload", json!({"days": 7}));
    assert_eq!(week["days"], 7);
    assert_eq!(week["reviews_per_day"].as_array().unwrap().len(), 7);
    assert_eq!(week["study_minutes"].as_array().unwrap().len(), 7);
}

#[test]
fn an_item_is_read_by_id_or_key_with_its_schedule_and_stacks_by_folder_or_title() {
    let vault = sample_vault("mcp-item-schedule");
    let a = vault.path.join("french/a.md");
    let text = std::fs::read_to_string(&a).unwrap();
    std::fs::write(
        &a,
        text.replace("kind: card\n", "kind: card\nkey: FLASHCARD_1\n"),
    )
    .unwrap();
    let mut c = ctx(&vault.path);
    with_schedule(&c);
    let answer = read(&mut c, "studyfrog://items/flashcard_1");
    assert_eq!(answer["result"]["contents"][0]["mimeType"], "text/markdown");
    let schedule = content(&answer, 1);
    let first = &schedule["schedules"][0];
    assert_eq!(first["folder"], "french");
    assert_eq!(first["state"]["algorithm"], "fsrs");
    assert_eq!(first["state"]["stability_days"], 3.5);
    assert_eq!(first["due_ms"], 1_700_000_000_000_i64);
    let by_title = content(&read(&mut c, "studyfrog://stacks/french"), 0);
    assert_eq!(by_title["title"], "French");
    assert_eq!(by_title["items"].as_array().unwrap().len(), 3);
    let summary = content(&read(&mut c, "studyfrog://vault/summary"), 0);
    assert_eq!(summary["items_by_kind"]["card"], 2);
    assert!(summary["streak_days"].is_u64());
}

#[test]
fn the_due_queue_lists_what_a_spaced_run_would_show_for_a_stack() {
    let vault = sample_vault("mcp-queue");
    let mut c = ctx(&vault.path);
    let p = McpPolicy::default();
    let all = call(&mut c, p, "vault_get_due_queue", json!({}));
    assert_eq!(all["queued"], 4, "every new item: {all}");
    let french = call(&mut c, p, "vault_get_due_queue", json!({"stack": "French"}));
    assert_eq!(french["queued"], 3, "the new items of French only");
    let none = call(&mut c, p, "vault_get_due_queue", json!({"stack": "Latin"}));
    assert!(none.as_str().unwrap().contains("no stack Latin"));
}

#[test]
fn the_diary_of_a_day_lists_its_entries_with_their_ratings() {
    let vault = sample_vault("mcp-diary");
    std::fs::create_dir_all(vault.path.join("diary")).unwrap();
    std::fs::write(
        vault.path.join("diary/2026-11-07.md"),
        format!(
            "---\nid: {}\nkind: diary\ndate: 2026-11-07\nstudy_minutes: 40\nfocus: 4\n---\nGood day.\n",
            format_ulid(Uid(50))
        ),
    )
    .unwrap();
    let mut c = ctx(&vault.path);
    let day = content(&read(&mut c, "studyfrog://diary/2026-11-07"), 0);
    let entry = &day["entries"][0];
    assert_eq!(entry["study_minutes"], 40);
    assert_eq!(entry["ratings"]["focus"], 4);
    assert!(entry["text"].as_str().unwrap().contains("Good day."));
    let empty = content(&read(&mut c, "studyfrog://diary/2026-11-08"), 0);
    assert_eq!(empty["entries"], json!([]));
    assert_eq!(
        read(&mut c, "studyfrog://diary/yesterday")["error"]["code"],
        -32602
    );
}

#[test]
fn pdf_pages_come_with_their_text_and_nothing_anywhere_changes() {
    let vault = sample_vault("mcp-pdf");
    std::fs::write(vault.path.join("french/three.pdf"), pdfmake::three()).unwrap();
    let mut c = ctx(&vault.path);
    studyfrog_service::pdfs::index_pdfs_fully(&mut c.session, i64::MAX / 2).unwrap();
    with_schedule(&c);
    let (files, rows) = (markdown_files(&vault.path), index_rows(&c));
    let p = McpPolicy::default();
    let pages = call(
        &mut c,
        p,
        "pdf_read_pages",
        json!({"path": "french/three.pdf", "first_page": 2, "last_page": 9}),
    );
    let list = pages["pages"].as_array().unwrap();
    assert_eq!(list.len(), 2, "pages 2 and 3 of 3");
    assert!(list[0]["text"].as_str().unwrap().contains("Beta"));
    let missing = call(&mut c, p, "pdf_read_pages", json!({"path": "nope.pdf"}));
    assert!(missing.as_str().unwrap().contains("not a PDF"));
    for tool in [
        "vault_get_due_queue",
        "analytics_get_report",
        "forecast_simulate_workload",
    ] {
        call(&mut c, p, tool, json!({}));
    }
    for uri in [
        "studyfrog://reports/familiarity",
        "studyfrog://reports/runs",
        "studyfrog://diary/2026-11-07",
        "studyfrog://stacks/French",
    ] {
        read(&mut c, uri);
    }
    read(
        &mut c,
        &format!("studyfrog://items/{}", format_ulid(Uid(10))),
    );
    assert_eq!(markdown_files(&vault.path), files);
    assert_eq!(index_rows(&c), rows, "no row of the index changed");
}
