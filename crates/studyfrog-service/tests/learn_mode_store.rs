#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Learn modes in the vault (STORY-0198): a valid graph round-trips through its file; an invalid one is not
//! written, and the reasons come back.

use std::fs;
use std::path::PathBuf;

use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind};
use studyfrog_service::learn_mode_store::{
    FOLDER, file_name, from_toml, list, load, save, to_toml,
};

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-learn-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    Dir(dir)
}

fn interleave() -> LearnGraph {
    let node = |id: &str, kind| Node {
        id: id.into(),
        kind,
        label: format!("{id} label"),
    };
    LearnGraph {
        name: "Interleave two decks".into(),
        nodes: vec![
            node("start", NodeKind::Start),
            node("show", NodeKind::Present),
            node("rate", NodeKind::Rate),
            node("pick", NodeKind::Branch),
            node("next", NodeKind::Schedule),
            node("end", NodeKind::Stop),
        ],
        edges: vec![
            Edge {
                from: "start".into(),
                to: "show".into(),
                when: None,
            },
            Edge {
                from: "show".into(),
                to: "rate".into(),
                when: None,
            },
            Edge {
                from: "rate".into(),
                to: "pick".into(),
                when: None,
            },
            Edge {
                from: "pick".into(),
                to: "show".into(),
                when: Some("again".into()),
            },
            Edge {
                from: "pick".into(),
                to: "next".into(),
                when: Some("default".into()),
            },
            Edge {
                from: "next".into(),
                to: "end".into(),
                when: None,
            },
        ],
    }
}

#[test]
fn a_valid_mode_is_saved_in_the_vault_and_reads_back_the_same() {
    let d = temp("round");
    let g = interleave();
    let path = save(&d.0, &g).unwrap();
    assert_eq!(path, d.0.join(FOLDER).join("interleave-two-decks.toml"));
    assert_eq!(list(&d.0), vec!["interleave-two-decks.toml".to_owned()]);
    assert_eq!(load(&d.0, "interleave-two-decks.toml").unwrap(), g);
}

#[test]
fn an_invalid_mode_is_not_saved_and_the_reasons_come_back() {
    let d = temp("refused");
    let mut g = interleave();
    g.nodes.retain(|n| n.kind != NodeKind::Start);
    let err = save(&d.0, &g).unwrap_err();
    assert!(err.contains("no start node"), "{err}");
    assert!(list(&d.0).is_empty(), "nothing was written");
}

#[test]
fn an_unknown_node_kind_in_a_file_is_an_error() {
    let text = "name = \"x\"\n\n[[node]]\nid = \"a\"\nkind = \"dance\"\n";
    assert!(
        from_toml(text)
            .unwrap_err()
            .contains("unknown node kind dance")
    );
}

#[test]
fn file_names_are_slugs() {
    assert_eq!(file_name("  My Mode!  "), "my-mode.toml");
    assert_eq!(file_name("!!!"), "mode.toml");
}

#[test]
fn the_text_of_a_graph_round_trips() {
    let g = interleave();
    assert_eq!(from_toml(&to_toml(&g).unwrap()).unwrap(), g);
}
