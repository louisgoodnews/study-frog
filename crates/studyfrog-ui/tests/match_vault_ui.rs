#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless UI and worker integration tests for vault-sourced Match mode (EPIC-075, TASK-0304).

mod common;

use std::fs;
use std::path::Path;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::{Effect, EffectVault};
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::match_ui::MatchMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{Msg, MsgVault};
use studyfrog_ui::state::Screen;

fn write_file(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

fn card(id: &str, title: &str, front: &str, back: &str) -> String {
    format!("---\nid: {id}\nkind: card\ntitle: {title}\n---\n{front}\n???\n{back}\n")
}

fn run_effects(
    model: &mut Model,
    worker: &mut studyfrog_ui::exec::Worker,
    effects: Vec<studyfrog_ui::effect::Effect>,
) {
    let mut queue = effects;
    while !queue.is_empty() {
        let mut next = Vec::new();
        for fx in queue {
            for msg in execute(worker, fx, 1_700_000_000_000_000_000) {
                next.extend(apply(model, msg));
            }
        }
        queue = next;
    }
}

#[test]
fn opening_match_loads_real_vault_pairs_and_never_biology_demo_cards() {
    let dir = std::env::temp_dir().join(format!(
        "sf_match_ui_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    write_file(
        &dir,
        "diag/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Diagnostik\n---\n",
    );
    for i in 1..=8u8 {
        let id = format!("0000000000000000000000001{i}");
        write_file(
            &dir,
            &format!("diag/c{i}.md"),
            &card(
                &id,
                &format!("Begriff {i}"),
                &format!("Frage Nummer {i}"),
                &format!("Antwort Nummer {i}"),
            ),
        );
    }

    let mut worker = worker_new(None);
    let mut model = Model::default();
    model.state.app.tutorial_offered = true;
    run_effects(
        &mut model,
        &mut worker,
        vec![Effect::Vault(EffectVault::OpenVault(
            dir.to_str().unwrap().into(),
        ))],
    );

    let nav_fx = apply(&mut model, Msg::Vault(MsgVault::Navigate(Screen::Match)));
    assert!(
        !nav_fx.is_empty(),
        "navigating to Match should dispatch LoadPairs"
    );
    run_effects(&mut model, &mut worker, nav_fx);

    let ms = model
        .state
        .match_state
        .as_ref()
        .expect("match_state loaded");
    assert_eq!(ms.pool.len(), 8);
    assert_eq!(ms.pairs_source.len(), 6);
    assert!(
        !ms.pairs_source.iter().any(|(p, _)| p == "Mitochondria"),
        "live vault must never fall back to biology demo cards"
    );

    let first_round = ms.pairs_source.clone();
    let _ = apply(&mut model, Msg::Match(MatchMsg::NewRound));
    let second_round = model
        .state
        .match_state
        .as_ref()
        .unwrap()
        .pairs_source
        .clone();
    assert_eq!(second_round.len(), 6);
    assert_ne!(
        first_round, second_round,
        "NewRound should deal a fresh subset"
    );

    let mut h = harness(model);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run_steps(4);
    assert!(h.query_by_label("Mitochondria").is_none());
    assert!(h.query_by_label("New round").is_some());
    assert!(h.query_by_label("Restart").is_some());
    assert!(h.query_by_label("Exit").is_some());
    assert_all_named(&h, "match screen with real vault pairs");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn empty_vault_scope_shows_honest_empty_state_instead_of_demo_cards() {
    let mut m = open_model();
    let _ = apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Match)));
    let _ = apply(&mut m, Msg::Match(MatchMsg::PairsLoaded(Ok(Vec::new()))));

    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run_steps(4);

    assert!(
        h.query_by_label("Not enough cards or questions to match")
            .is_some()
    );
    assert!(h.query_by_label("Mitochondria").is_none());
    assert_all_named(&h, "match screen empty state");
}
