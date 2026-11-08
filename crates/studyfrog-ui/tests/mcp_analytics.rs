#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The MCP reports of EPIC-034 (STORY-0118): leeches, study minutes and true retention, read-only.

mod common;
mod mcp_common;

use mcp_common::*;
use studyfrog_core::review::Rating::{self, Again, Good};
use studyfrog_core::time::UnixMs;

const DAY: i64 = 86_400_000;

/// Card A (10) forgotten four times after its first review, B (11) remembered.
fn with_history(c: &VaultCtx) {
    let p = studyfrog_srs::params::params_default();
    let t0 = now() - 60 * DAY;
    let plan: [(u128, i64, Rating); 9] = [
        (10, 0, Good),
        (11, 0, Good),
        (10, 2, Again),
        (10, 3, Again),
        (11, 4, Good),
        (10, 5, Again),
        (10, 6, Again),
        (11, 20, Good),
        (10, 21, Good),
    ];
    let mut prev = std::collections::HashMap::new();
    for (item, day, rating) in plan {
        let (r, mem) = studyfrog_service::schedule::apply_rating(
            prev.get(&item),
            Uid(item),
            rating,
            UnixMs(t0 + day * DAY),
            &p,
        );
        studyfrog_store::reviews::insert_review(&c.session.conn, &r).unwrap();
        studyfrog_store::schedule::save_mem_state(&c.session.conn, Uid(item), &mem).unwrap();
        prev.insert(item, mem);
    }
}

#[test]
fn leeches_are_a_resource_and_a_report() {
    let vault = sample_vault("mcp-leeches");
    let mut c = ctx(&vault.path);
    with_history(&c);
    let resource = content(&read(&mut c, "studyfrog://reports/leeches"), 0);
    let leeches = resource["leeches"].as_array().unwrap();
    assert_eq!(leeches.len(), 1, "{resource}");
    assert_eq!(leeches[0]["id"], format_ulid(Uid(10)));
    assert_eq!(leeches[0]["title"], "A");
    assert_eq!(leeches[0]["lapses"], 4);
    assert_eq!(leeches[0]["reasons"][0], "RepeatedLapses");
    assert!(
        resource["confusion_pairs"].as_array().unwrap().is_empty(),
        "A and B share no words, tags or links"
    );
    let report = call(
        &mut c,
        McpPolicy::default(),
        "analytics_get_report",
        json!({"kind": "leeches"}),
    );
    assert_eq!(report["kind"], "leeches");
    assert_eq!(report["report"], resource);
}

#[test]
fn the_workload_has_study_minutes_stages_and_true_retention() {
    let vault = sample_vault("mcp-minutes");
    let mut c = ctx(&vault.path);
    with_history(&c);
    let p = McpPolicy::default();
    let week = call(&mut c, p, "forecast_simulate_workload", json!({"days": 7}));
    assert_eq!(week["days"], 7);
    let minutes = week["study_minutes"].as_array().unwrap();
    assert_eq!(minutes.len(), 7);
    assert!(minutes.iter().all(|m| m.as_f64().unwrap() >= 0.0));
    let total: f64 = week["reviews_per_day"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .sum();
    assert!(total > 0.0, "the two cards come back: {week}");
    assert!(week["total_minutes"].as_f64().unwrap() > 0.0);
    let day = &week["per_day"][0];
    for key in [
        "new", "learning", "young", "mature", "p10", "p50", "p90", "minutes",
    ] {
        assert!(day[key].is_number(), "{key}: {day}");
    }
    assert!(week["true_retention"]["reviews"].is_u64());
    let year = call(
        &mut c,
        p,
        "forecast_simulate_workload",
        json!({"days": 365}),
    );
    assert_eq!(year["study_minutes"].as_array().unwrap().len(), 365);
}

#[test]
fn the_retention_report_carries_true_retention_and_the_tier_curves() {
    let vault = sample_vault("mcp-true-retention");
    let mut c = ctx(&vault.path);
    with_history(&c);
    let r = call(
        &mut c,
        McpPolicy::default(),
        "analytics_get_report",
        json!({"kind": "retention"}),
    );
    let t = &r["report"]["true_retention"];
    assert_eq!(t["stability_tiers"].as_array().unwrap().len(), 4);
    let young = &t["young"];
    assert_eq!(
        young["reviews"], 7,
        "every spaced review of A and B is young"
    );
    assert!(young["predicted"].as_f64().unwrap() > 0.0);
    let bins = t["stability_tiers"][0]["by_elapsed_over_stability"]
        .as_array()
        .unwrap();
    assert_eq!(bins.len(), 5);
    assert!(t["model_half_life_days"].is_number());
    let resource = content(&read(&mut c, "studyfrog://reports/retention"), 0);
    assert_eq!(resource["true_retention"], *t);
}

#[test]
fn the_new_reports_change_nothing() {
    let vault = sample_vault("mcp-analytics-readonly");
    let mut c = ctx(&vault.path);
    with_history(&c);
    let (files, rows) = (markdown_files(&vault.path), index_rows(&c));
    let p = McpPolicy::default();
    for kind in ["leeches", "retention", "forecast"] {
        call(&mut c, p, "analytics_get_report", json!({"kind": kind}));
        read(&mut c, &format!("studyfrog://reports/{kind}"));
    }
    call(
        &mut c,
        p,
        "forecast_simulate_workload",
        json!({"days": 365}),
    );
    assert_eq!(markdown_files(&vault.path), files);
    assert_eq!(index_rows(&c), rows, "no row of the index changed");
}
