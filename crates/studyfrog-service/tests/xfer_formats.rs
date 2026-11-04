#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Interchange export formats: JSON, TOML and RON (TASK-0220).

use std::fs;
use std::path::PathBuf;

use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_service::xfer_adapters::read_source;
use studyfrog_service::xfer_doc::ExportDoc;
use studyfrog_service::xfer_export::export_bundle;
use studyfrog_service::xfer_json::{from_json, to_json};
use studyfrog_service::xfer_ron::{from_ron, to_ron};
use studyfrog_service::xfer_toml::{from_toml, to_toml};
use studyfrog_store::schedule::insert_review;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn source(label: &str) -> (Dir, VaultSession) {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-formats-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let w = |p: &str, t: &str| {
        let f = dir.join(p);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, t).unwrap();
    };
    w(
        "french/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\n---\nAll about French.\n",
    );
    w(
        "french/verbs/_stack.md",
        "---\nid: 00000000000000000000000002\nkind: stack\ntitle: Verbs\n---\n",
    );
    w(
        "french/card.md",
        "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Bonjour\ntags: [greeting]\nreferenced_by: [00000000000000000000000002]\nstatus: learning\n---\nHello\n???\nBonjour\n",
    );
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

#[test]
fn plain_export_produces_identical_bundles_across_json_toml_and_ron() {
    let (_d, s) = source("plain");
    let bundle = export_bundle(&s, None, false).unwrap();
    let exported = "2026-01-01T00:00:00Z";

    let json_text = to_json(&bundle, exported);
    let toml_text = to_toml(&bundle, exported);
    let ron_text = to_ron(&bundle, exported);

    // Standard crates can parse them as valid data
    let _: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    let _: toml::Value = toml::from_str(&toml_text).unwrap();
    let _: ron::Value = ron::from_str(&ron_text).unwrap();

    // ExportDoc round trips
    let json_doc: ExportDoc = serde_json::from_str(&json_text).unwrap();
    let toml_doc: ExportDoc = toml::from_str(&toml_text).unwrap();
    let ron_doc: ExportDoc = ron::from_str(&ron_text).unwrap();
    assert_eq!(json_doc, toml_doc);
    assert_eq!(json_doc, ron_doc);

    // From implementations reconstruct identical bundles
    let from_j = from_json(&json_text).unwrap();
    let from_t = from_toml(&toml_text).unwrap();
    let from_r = from_ron(&ron_text).unwrap();

    assert_eq!(from_j.stacks, bundle.stacks);
    assert_eq!(from_j.items, bundle.items);
    assert_eq!(from_j, from_t);
    assert_eq!(from_j, from_r);
}

#[test]
fn advanced_export_includes_references_per_stack_values_averages_and_reviews() {
    let (_d, s) = source("advanced");

    // Add review history
    let r = Review {
        item: Uid(0xA),
        rating: Rating::Good,
        at: UnixMs(1_700_000_000_000),
        elapsed_days: 1.0,
    };
    insert_review(&s.conn, &r).unwrap();

    let bundle = export_bundle(&s, None, true).unwrap();
    let exported = "2026-01-01T00:00:00Z";

    let item = bundle
        .items
        .iter()
        .find(|i| i.id == Some(Uid(0xA)))
        .unwrap();
    assert_eq!(item.also_in, vec!["french/verbs"]);
    assert_eq!(item.reviews.len(), 1);

    let json_text = to_json(&bundle, exported);
    let toml_text = to_toml(&bundle, exported);
    let ron_text = to_ron(&bundle, exported);

    // All contain the advanced scheduling keys
    assert!(json_text.contains("\"reviews\"") && json_text.contains("\"also_in\""));
    assert!(toml_text.contains("reviews") && toml_text.contains("also_in"));
    assert!(ron_text.contains("reviews") && ron_text.contains("also_in"));

    // Standard crates parse without error
    let _: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    let _: toml::Value = toml::from_str(&toml_text).unwrap();
    let _: ron::Value = ron::from_str(&ron_text).unwrap();

    // Reconstruct identical bundles
    let from_j = from_json(&json_text).unwrap();
    let from_t = from_toml(&toml_text).unwrap();
    let from_r = from_ron(&ron_text).unwrap();

    assert_eq!(from_j.items[0].reviews, item.reviews);
    assert_eq!(from_j.items[0].also_in, item.also_in);
    assert_eq!(from_j, from_t);
    assert_eq!(from_j, from_r);
}

#[test]
fn read_source_supports_toml_and_ron_files() {
    let (_d, s) = source("read-source");
    let bundle = export_bundle(&s, None, false).unwrap();
    let tmp = std::env::temp_dir().join(format!("read-src-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let toml_path = tmp.join("test.toml");
    let ron_path = tmp.join("test.ron");
    fs::write(&toml_path, to_toml(&bundle, "2026-01-01T00:00:00Z")).unwrap();
    fs::write(&ron_path, to_ron(&bundle, "2026-01-01T00:00:00Z")).unwrap();

    let (from_t, adapter_t) = read_source(&toml_path).unwrap();
    let (from_r, adapter_r) = read_source(&ron_path).unwrap();

    assert_eq!(adapter_t, "StudyFrog export (TOML)");
    assert_eq!(adapter_r, "StudyFrog export (RON)");
    assert_eq!(from_t.items.len(), bundle.items.len());
    assert_eq!(from_r.items.len(), bundle.items.len());

    let _ = fs::remove_dir_all(&tmp);
}
