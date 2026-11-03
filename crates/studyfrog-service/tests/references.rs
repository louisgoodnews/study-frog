#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0212: the services follow the membership. An item referenced by a second stack counts there, is queued once in
//! a run, shows in that stack's report and library column, and travels through JSON, CSV and an import.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::plan::{RunPlan, plan_sequential};
use studyfrog_core::run::RunMode;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::create::IdSource;
use studyfrog_service::explore::{Period, Scope, explore};
use studyfrog_service::library_rows::load_library_rows;
use studyfrog_service::queue::{NewRun, new_run};
use studyfrog_service::session::open_vault;
use studyfrog_service::stack_summary::stack_summary;
use studyfrog_service::xfer_csv::to_csv;
use studyfrog_service::xfer_export::export_bundle;
use studyfrog_service::xfer_import::{Policy, apply_import};
use studyfrog_service::xfer_json::{from_json, to_json};

const NOW: i64 = 1_800_000_000_000;
const STACK_A: &str = "00000000000000000000000001";
const STACK_B: &str = "00000000000000000000000002";

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str, referenced: bool) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-refs-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("a")).unwrap();
    fs::create_dir_all(dir.join("b")).unwrap();
    let w = |p: &str, t: String| fs::write(dir.join(p), t).unwrap();
    w(
        "a/_stack.md",
        format!("---\nid: {STACK_A}\nkind: stack\ntitle: Alpha\n---\n"),
    );
    w(
        "b/_stack.md",
        format!("---\nid: {STACK_B}\nkind: stack\ntitle: Beta\n---\n"),
    );
    let by = if referenced {
        format!("referenced_by: [{STACK_B}]\n")
    } else {
        String::new()
    };
    w(
        "a/shared.md",
        format!(
            "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Shared\n{by}---\nq\n???\na\n"
        ),
    );
    w(
        "a/own.md",
        "---\nid: 0000000000000000000000000B\nkind: card\ntitle: Own A\n---\nq2\n???\na2\n"
            .to_owned(),
    );
    w(
        "b/own.md",
        "---\nid: 0000000000000000000000000C\nkind: question\ntitle: Own B\n---\nq3\n???\na3\n"
            .to_owned(),
    );
    Dir(dir)
}

fn run_of(session: &studyfrog_service::session::VaultSession, stacks: &[u128]) -> Vec<Uid> {
    let mut plan: RunPlan = plan_sequential();
    plan.filter.stacks = stacks.iter().map(|s| Uid(*s)).collect();
    let day = studyfrog_service::day::day_from(UnixMs(NOW - NOW.rem_euclid(86_400_000)));
    let req = NewRun {
        plan,
        mode: RunMode::Flip,
        uid: Uid(99),
        now: UnixMs(NOW),
        day,
        global_budget: studyfrog_rehearsal::queue::candidate::budget_unlimited(),
    };
    new_run(
        &session.conn,
        req,
        &studyfrog_core::settings::vault_settings_default(),
    )
    .unwrap()
    .queue
}

#[test]
fn a_referenced_item_counts_in_the_summary_of_every_stack_that_holds_it() {
    let v = vault("summary", true);
    let (s, _) = open_vault(v.0.to_str().unwrap()).unwrap();
    let a = stack_summary(&s.conn, "a", UnixMs(NOW)).unwrap();
    let b = stack_summary(&s.conn, "b", UnixMs(NOW)).unwrap();
    assert_eq!(a.types, [0, 2, 0], "Alpha: its two cards");
    assert_eq!(
        b.types,
        [0, 1, 1],
        "Beta: its question and the card it references"
    );
}

#[test]
fn a_run_over_a_stack_includes_what_it_references_and_a_run_over_both_queues_it_once() {
    let v = vault("run", true);
    let (s, _) = open_vault(v.0.to_str().unwrap()).unwrap();
    let only_b = run_of(&s, &[2]);
    assert!(
        only_b.contains(&Uid(0xA)) && only_b.contains(&Uid(0xC)) && !only_b.contains(&Uid(0xB))
    );
    let only_a = run_of(&s, &[1]);
    assert!(
        only_a.contains(&Uid(0xA)) && only_a.contains(&Uid(0xB)) && !only_a.contains(&Uid(0xC))
    );
    let both = run_of(&s, &[1, 2]);
    assert_eq!(
        both.iter().filter(|u| **u == Uid(0xA)).count(),
        1,
        "queued once"
    );
    assert_eq!(both.len(), 3);
}

#[test]
fn reports_and_the_library_show_the_second_stack() {
    let v = vault("rows", true);
    let (s, _) = open_vault(v.0.to_str().unwrap()).unwrap();
    let rows = load_library_rows(&s, UnixMs(NOW)).unwrap();
    let shared = rows.iter().find(|r| r.uid == Uid(0xA)).unwrap();
    assert_eq!(shared.row.also_in, ["Beta"]);
    assert_eq!(
        shared.row.stacks.last().map(String::as_str),
        Some("Alpha"),
        "its primary stack stays"
    );
    let own = rows.iter().find(|r| r.uid == Uid(0xB)).unwrap();
    assert!(own.row.also_in.is_empty());
    let report = explore(&s, (Scope::Stack("b".into()), Period::All), UnixMs(NOW), 0).unwrap();
    let titles: Vec<&str> = report.rows.iter().map(|r| r.label.as_str()).collect();
    assert!(
        titles.contains(&"Own B") && titles.contains(&"Shared"),
        "{titles:?}"
    );
}

#[test]
fn a_vault_without_references_behaves_exactly_as_the_folders_say() {
    let v = vault("plain", false);
    let (s, _) = open_vault(v.0.to_str().unwrap()).unwrap();
    assert_eq!(
        stack_summary(&s.conn, "a", UnixMs(NOW)).unwrap().types,
        [0, 2, 0]
    );
    assert_eq!(
        stack_summary(&s.conn, "b", UnixMs(NOW)).unwrap().types,
        [0, 0, 1]
    );
    assert_eq!(run_of(&s, &[2]), [Uid(0xC)]);
    assert_eq!(run_of(&s, &[1]).len(), 2);
    assert!(
        load_library_rows(&s, UnixMs(NOW))
            .unwrap()
            .iter()
            .all(|r| r.row.also_in.is_empty())
    );
}

#[test]
fn json_and_csv_carry_the_references_and_a_scoped_export_keeps_only_those_in_scope() {
    let v = vault("export", true);
    let (s, _) = open_vault(v.0.to_str().unwrap()).unwrap();
    let all = export_bundle(&s, None, false).unwrap();
    let shared = all.items.iter().find(|i| i.title == "Shared").unwrap();
    assert_eq!(shared.also_in, ["b"]);
    let back = from_json(&to_json(&all, "2026-01-01T00:00:00Z")).unwrap();
    assert_eq!(
        back.items
            .iter()
            .find(|i| i.title == "Shared")
            .unwrap()
            .also_in,
        ["b"]
    );
    let csv = to_csv(&all);
    assert!(csv.lines().next().unwrap().ends_with(",also_in"), "{csv}");
    let read = studyfrog_service::xfer_csv::from_csv(&csv).unwrap();
    assert_eq!(
        read.items
            .iter()
            .find(|i| i.title == "Shared")
            .unwrap()
            .also_in,
        ["b"]
    );
    let only_a = export_bundle(&s, Some("a"), false).unwrap();
    assert!(
        only_a.items.iter().all(|i| i.also_in.is_empty()),
        "Beta is outside the export"
    );
    let plain = vault("export-plain", false);
    let (p, _) = open_vault(plain.0.to_str().unwrap()).unwrap();
    let csv = to_csv(&export_bundle(&p, None, false).unwrap());
    assert!(!csv.contains("also_in"), "no references, no extra column");
}

#[test]
fn an_import_turns_also_in_into_referenced_by_and_creates_the_stack_it_names() {
    let from = vault("import-from", true);
    let (src, _) = open_vault(from.0.to_str().unwrap()).unwrap();
    let bundle = export_bundle(&src, None, false).unwrap();
    let into =
        std::env::temp_dir().join(format!("studyfrog-refs-import-into-{}", std::process::id()));
    let _ = fs::remove_dir_all(&into);
    fs::create_dir_all(&into).unwrap();
    let _guard = Dir(into.clone());
    let (mut dst, _) = open_vault(into.to_str().unwrap()).unwrap();
    let ids = IdSource {
        now_ms: 1_800_000_000_000,
        random: [5; 10],
    };
    let done = apply_import(&mut dst, &bundle, None, (Policy::Skip, false), ids).unwrap();
    assert_eq!(done.created, 3, "{:?}", done.notes);
    let rows = load_library_rows(&dst, UnixMs(NOW)).unwrap();
    let shared = rows.iter().find(|r| r.row.title == "Shared").unwrap();
    assert_eq!(shared.row.also_in, ["Beta"], "{:?}", done.notes);
    let text = fs::read_to_string(into.join(&shared.row.path)).unwrap();
    assert!(text.contains("referenced_by: ["), "{text}");
}
