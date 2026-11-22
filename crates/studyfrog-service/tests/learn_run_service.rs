#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A custom learn-mode run (STORY-0198): it starts from a saved, checked mode; each rating walks the graph and
//! is stored as steps; it writes no review; an invalid mode is refused at start.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::plan::plan_spaced;
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind};
use studyfrog_service::create::IdSource;
use studyfrog_service::day::DayWindow;
use studyfrog_service::learn_mode_store::save;
use studyfrog_service::learn_run::{record_rating, start};
use studyfrog_service::session::open_vault;
use studyfrog_store::learn::item_path;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn node(id: &str, kind: NodeKind) -> Node {
    Node {
        id: id.into(),
        kind,
        label: id.into(),
    }
}

fn edge(from: &str, to: &str, when: Option<&str>) -> Edge {
    Edge {
        from: from.into(),
        to: to.into(),
        when: when.map(str::to_owned),
    }
}

fn mode() -> LearnGraph {
    LearnGraph {
        name: "Interleave".into(),
        nodes: vec![
            node("start", NodeKind::Start),
            node("show", NodeKind::Present),
            node("rate", NodeKind::Rate),
            node("pick", NodeKind::Branch),
            node("next", NodeKind::Schedule),
            node("end", NodeKind::Stop),
        ],
        edges: vec![
            edge("start", "show", None),
            edge("show", "rate", None),
            edge("rate", "pick", None),
            edge("pick", "show", Some("again")),
            edge("pick", "next", Some("default")),
            edge("next", "end", None),
        ],
    }
}

fn now() -> UnixMs {
    UnixMs(1_700_000_000_000)
}

fn ids() -> IdSource {
    IdSource {
        now_ms: 1_700_000_000_000,
        random: [7; 10],
    }
}

fn day() -> DayWindow {
    DayWindow {
        start: UnixMs(1_699_990_000_000),
        end: UnixMs(1_700_080_000_000),
    }
}

#[test]
fn a_custom_run_walks_its_graph_and_writes_no_review() {
    let d = Dir(std::env::temp_dir().join(format!("studyfrog-learn-run-{}", std::process::id())));
    let _ = fs::remove_dir_all(&d.0);
    fs::create_dir_all(&d.0).unwrap();
    let (session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    save(&d.0, &mode()).unwrap();

    let run = start(
        &session,
        "interleave.toml",
        plan_spaced(),
        now(),
        day(),
        ids(),
    )
    .unwrap();
    let run_id = format_ulid(run.uid);
    let item = format_ulid(Uid(77));
    let back = record_rating(&session.conn, run.uid, Uid(77), Rating::Again, now()).unwrap();
    assert_eq!(
        back,
        Some(true),
        "Again loops back: the item is shown again in this run"
    );
    let path = item_path(&session.conn, &run_id, &item).unwrap();
    let nodes: Vec<&str> = path.iter().map(|p| p.0.as_str()).collect();
    assert_eq!(nodes, ["show", "rate", "pick", "show", "rate"], "{nodes:?}");
    assert_eq!(
        path[1].2.as_deref(),
        Some("again"),
        "the rating is on the rate node it left"
    );

    let out = record_rating(&session.conn, run.uid, Uid(77), Rating::Good, now()).unwrap();
    assert_eq!(out, Some(false), "Good takes the item out of the run");

    let reviews: i64 = session
        .conn
        .query_row("SELECT COUNT(*) FROM review", [], |r| r.get(0))
        .unwrap();
    assert_eq!(reviews, 0, "a custom run is practice-only");
}

#[test]
fn an_invalid_mode_file_is_refused_at_start() {
    let d = Dir(std::env::temp_dir().join(format!("studyfrog-learn-bad-{}", std::process::id())));
    let _ = fs::remove_dir_all(&d.0);
    fs::create_dir_all(d.0.join("learn_modes")).unwrap();
    let (session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    fs::write(
        d.0.join("learn_modes/broken.toml"),
        "name = \"broken\"\n\n[[node]]\nid = \"show\"\nkind = \"present\"\n",
    )
    .unwrap();
    let err = start(&session, "broken.toml", plan_spaced(), now(), day(), ids()).unwrap_err();
    assert!(err.to_string().contains("not run"), "{err}");
}
