#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The graph screen: pure update logic, the real-vault flow, and the real widgets.
use common::views::{assert_all_named, harness, open_model};
use common::{run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use std::fs;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_store::overview::{GraphNode, LinkGraph};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectEditor;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::msg::MsgGraph;
use studyfrog_ui::msg::MsgVault;
use studyfrog_ui::state::Screen;

mod common;

fn chain(n: u128) -> LinkGraph {
    let nodes = (0..n)
        .map(|i| GraphNode {
            uid: Uid(i + 1),
            title: format!("Note {i}"),
            kind: ItemKind::Note,
        })
        .collect();
    let edges = (0..n.saturating_sub(1))
        .map(|i| (i as u32, i as u32 + 1))
        .collect();
    LinkGraph { nodes, edges }
}

fn loaded(graph: LinkGraph) -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Graph(MsgGraph::GraphLoaded(Ok(graph))));
    m
}

#[test]
fn the_graph_reads_the_real_index_and_follows_the_vault_while_open() {
    let dir = temp_dir("graphflow");
    let w = |p: &str, t: &str| {
        fs::create_dir_all(dir.path.join(p).parent().unwrap()).unwrap();
        fs::write(dir.path.join(p), t).unwrap();
    };
    w(
        "s/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    );
    w(
        "s/a.md",
        "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Alpha\n---\nsee [[Beta]]\n",
    );
    w(
        "s/b.md",
        "---\nid: 0000000000000000000000000B\nkind: note\ntitle: Beta\n---\n",
    );
    let (mut m, mut worker) = (Model::default(), worker_new(None));
    let fx = apply(
        &mut m,
        Msg::Vault(MsgVault::OpenRecent(dir.path.display().to_string())),
    );
    run_to_idle(&mut m, &mut worker, fx);
    let fx = apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Graph)));
    run_to_idle(&mut m, &mut worker, fx);
    let g = m.state.graph.as_ref().unwrap();
    assert_eq!((g.graph.nodes.len(), g.graph.edges.len()), (2, 1));

    w(
        "s/c.md",
        "---\nid: 0000000000000000000000000C\nkind: note\ntitle: Gamma\n---\nsee [[alpha]]\n",
    );
    let fx = apply(&mut m, Msg::Vault(MsgVault::Tick));
    run_to_idle(&mut m, &mut worker, fx);
    let g = m.state.graph.as_ref().unwrap();
    assert_eq!(
        (g.graph.nodes.len(), g.graph.edges.len()),
        (3, 2),
        "the open graph picked up the new file"
    );

    let fx = apply(&mut m, Msg::Graph(MsgGraph::GraphClick(0)));
    run_to_idle(&mut m, &mut worker, fx);
    assert_eq!(
        (m.state.screen, m.state.editor.as_ref().map(|e| e.uid)),
        (Screen::Editor, Some(Uid(0xA)))
    );
}

#[test]
fn the_real_screen_shows_counts_offers_a_list_and_is_accessible() {
    let mut m = loaded(chain(4));
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Graph)));
    let mut h = harness(m);
    h.run_steps(3);
    assert!(
        h.query_by_label("4 items, 3 links").is_some()
            && h.query_by_label("Re-run layout").is_some()
    );
    assert!(
        h.state().0.state.graph.as_ref().unwrap().fitted,
        "the canvas reported its size"
    );
    assert_all_named(&h, "graph");

    h.get_by_label("Items as a list").click();
    h.run_steps(2);
    h.get_by_label("Note 2").click();
    h.run_steps(2);
    assert!(
        h.state()
            .1
            .contains(&Effect::Editor(EffectEditor::LoadDocument(Uid(3)))),
        "the accessible list opens items too"
    );
    let seed = h.state().0.state.graph.as_ref().unwrap().seed;
    h.get_by_label("Re-run layout").click();
    h.run_steps(2);
    assert_ne!(
        h.state().0.state.graph.as_ref().unwrap().seed,
        seed,
        "the button restarts the layout"
    );
}
