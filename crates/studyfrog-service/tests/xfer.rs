#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Export and import: formats, the round trip, the dry run, duplicates and hostile files.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::create::IdSource;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_service::xfer_export::{export_bundle, export_json_zip};
use studyfrog_service::xfer_import::{Policy, Verdict, apply_import, counts, plan_import};
use studyfrog_service::xfer_json::{from_json, to_json};
use studyfrog_service::xfer_zip::read_zip;
use studyfrog_store::schedule::{insert_review, load_mem_state};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-xfer-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn source(label: &str) -> (Dir, VaultSession) {
    let (d, _) = vault(label);
    let w = |p: &str, t: &str| {
        let f = d.0.join(p);
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
        "french/a.md",
        "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Bonjour\ntags: [greeting, \"a,b\"]\nstatus: learning\npriority: high\n---\nHello, \"world\"\n???\nBonjour\nle monde\n",
    );
    w(
        "french/verbs/b.md",
        "---\nid: 0000000000000000000000000B\nkind: question\ntitle: Etre\n---\nTo be?\n???\nEtre\n",
    );
    w(
        "french/n.md",
        "---\nid: 0000000000000000000000000C\nkind: note\ntitle: Note\n---\nA note [[Bonjour]]\n",
    );
    w(
        "maths/_stack.md",
        "---\nid: 00000000000000000000000003\nkind: stack\ntitle: Maths\n---\n",
    );
    w(
        "maths/d.md",
        "---\nid: 0000000000000000000000000D\nkind: card\ntitle: Primes\n---\n2\n???\nprime\n",
    );
    fs::write(d.0.join("french/pic.png"), [137, 80, 78, 71]).unwrap();
    let (s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    (d, s)
}

fn ids() -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000,
        random: [7; 10],
    }
}

fn titles(s: &VaultSession) -> Vec<String> {
    let mut t: Vec<String> = studyfrog_store::item_meta::load_item_meta(&s.conn)
        .unwrap()
        .values()
        .map(|m| m.title.clone())
        .collect();
    t.sort();
    t
}

#[test]
fn a_json_export_imports_into_an_empty_vault_with_the_same_items_and_files() {
    let (_a, s) = source("rt-src");
    let bundle = export_bundle(&s, None, false).unwrap();
    assert_eq!((bundle.stacks.len(), bundle.items.len()), (3, 4));
    let zip = export_json_zip(&bundle, "2026-11-03T00:00:00Z").unwrap();
    let files = read_zip(&zip).unwrap();
    assert!(files.iter().any(|(p, _)| p == "files/french/pic.png"));
    let json = String::from_utf8(
        files
            .iter()
            .find(|(p, _)| p == "export.json")
            .unwrap()
            .1
            .clone(),
    )
    .unwrap();
    let back = from_json(&json).unwrap();
    assert_eq!(
        back.items, bundle.items,
        "reading what was written gives the same items"
    );
    assert!(
        !json.contains("reviews"),
        "the default export has no scheduling"
    );

    let (d2, mut t) = vault("rt-dst");
    let mut b2 = from_json(&json).unwrap();
    b2.files = files
        .into_iter()
        .filter_map(|(p, d)| p.strip_prefix("files/").map(|r| (r.to_owned(), d)))
        .collect();
    let done = apply_import(&mut t, &b2, None, (Policy::Skip, false), ids()).unwrap();
    assert_eq!((done.created, done.skipped), (4, 0), "{:?}", done.notes);
    assert_eq!(titles(&t), titles(&s));
    assert!(d2.0.join("french/pic.png").is_file());
    let again = export_bundle(&t, None, false).unwrap();
    assert_eq!(
        again.items, bundle.items,
        "ids, text, tags, status and priority survive the trip"
    );
    assert!(
        again
            .stacks
            .iter()
            .any(|x| x.folder == "french" && x.description.contains("All about French"))
    );
}

#[test]
fn scheduling_travels_only_in_the_advanced_export_and_is_written_only_when_asked() {
    let (_a, s) = source("sched-src");
    for (i, rating) in [Rating::Good, Rating::Good, Rating::Easy]
        .into_iter()
        .enumerate()
    {
        let r = Review {
            item: Uid(0xA),
            rating,
            at: UnixMs(1_700_000_000_000 + i as i64 * 86_400_000 * 3),
            elapsed_days: 3.0,
        };
        insert_review(&s.conn, &r).unwrap();
    }
    let plain = to_json(&export_bundle(&s, None, false).unwrap(), "x");
    assert!(!plain.contains("\"reviews\""));
    let advanced = to_json(&export_bundle(&s, None, true).unwrap(), "x");
    let b = from_json(&advanced).unwrap();
    assert_eq!(
        b.items
            .iter()
            .find(|i| i.id == Some(Uid(0xA)))
            .unwrap()
            .reviews
            .len(),
        3
    );

    let (_d, mut t) = vault("sched-dst");
    apply_import(&mut t, &b, None, (Policy::Skip, false), ids()).unwrap();
    assert!(
        load_mem_state(&t.conn, Uid(0xA)).unwrap().is_none(),
        "left out when the box is not ticked"
    );
    let (_d2, mut u) = vault("sched-dst2");
    apply_import(&mut u, &b, None, (Policy::Skip, true), ids()).unwrap();
    assert!(
        load_mem_state(&u.conn, Uid(0xA)).unwrap().is_some(),
        "the schedule is rebuilt from the reviews"
    );
}

#[test]
fn the_dry_run_writes_nothing_and_duplicates_follow_the_chosen_policy() {
    let (_a, s) = source("dup-src");
    let mut b = export_bundle(&s, None, false).unwrap();
    let (_d, mut t) = vault("dup-dst");
    apply_import(&mut t, &b, None, (Policy::Skip, false), ids()).unwrap();
    let before = titles(&t);

    b.items[0].answer = "changed".to_owned();
    b.items.push(studyfrog_service::xfer_model::item(
        studyfrog_core::plan::ItemKind::Card,
        "maths",
        "",
        "x",
    ));
    let plan = plan_import(&t, &b, None).unwrap();
    assert_eq!(counts(&plan), (0, 4, 1), "{:?}", plan.lines);
    assert!(matches!(&plan.lines[0].verdict, Verdict::Duplicate(p) if p == "french/bonjour.md"));
    assert_eq!(titles(&t), before, "a dry run writes nothing");

    let skip = apply_import(&mut t, &b, None, (Policy::Skip, false), ids()).unwrap();
    assert_eq!((skip.created, skip.replaced, skip.skipped), (0, 0, 5));
    assert!(
        !fs::read_to_string(t.root.join("french/bonjour.md"))
            .unwrap()
            .contains("changed")
    );
    let replace = apply_import(&mut t, &b, None, (Policy::Replace, false), ids()).unwrap();
    assert_eq!((replace.created, replace.replaced), (0, 4));
    assert!(
        fs::read_to_string(t.root.join("french/bonjour.md"))
            .unwrap()
            .contains("changed")
    );
    let both = apply_import(&mut t, &b, None, (Policy::KeepBoth, false), ids()).unwrap();
    assert_eq!(both.created, 4);
    assert_eq!(titles(&t).len(), before.len() + 4);
}
