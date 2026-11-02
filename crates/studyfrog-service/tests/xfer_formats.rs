#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! CSV and Markdown exports.

use std::fs;

use studyfrog_service::session::open_vault;
use studyfrog_service::xfer_csv::{from_csv, to_csv};
use studyfrog_service::xfer_export::{export_bundle, export_markdown_zip};
use studyfrog_service::xfer_zip::read_zip;

fn source(label: &str) -> studyfrog_service::session::VaultSession {
    let dir = std::env::temp_dir().join(format!("studyfrog-xfmt-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (p, t) in [
        (
            "french/_stack.md",
            "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\n---\n",
        ),
        (
            "french/verbs/_stack.md",
            "---\nid: 00000000000000000000000002\nkind: stack\ntitle: Verbs\n---\n",
        ),
        (
            "french/a.md",
            "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Bonjour\ntags: [greeting, \"a,b\"]\n---\nHello, \"world\"\n???\nBonjour\nle monde\n",
        ),
        (
            "french/verbs/b.md",
            "---\nid: 0000000000000000000000000B\nkind: question\ntitle: Etre\n---\nTo be?\n???\nEtre\n",
        ),
        (
            "french/n.md",
            "---\nid: 0000000000000000000000000C\nkind: note\ntitle: Note\n---\nA note\n",
        ),
        (
            "maths/_stack.md",
            "---\nid: 00000000000000000000000003\nkind: stack\ntitle: Maths\n---\n",
        ),
        (
            "maths/d.md",
            "---\nid: 0000000000000000000000000D\nkind: card\ntitle: Primes\n---\n2\n???\nprime\n",
        ),
    ] {
        let f = dir.join(p);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, t).unwrap();
    }
    fs::write(dir.join("french/pic.png"), [137, 80, 78, 71]).unwrap();
    open_vault(dir.to_str().unwrap()).unwrap().0
}

#[test]
fn csv_round_trips_with_quotes_commas_and_line_breaks_and_markdown_exports_the_vault_files() {
    let s = source("csv");
    let b = export_bundle(&s, Some("french"), false).unwrap();
    assert_eq!(b.items.len(), 3, "the stack and the stack inside it");
    let csv = to_csv(&b);
    let back = from_csv(&csv).unwrap();
    assert_eq!(back.items.len(), 2, "the note is not in a CSV");
    let bonjour = back.items.iter().find(|i| i.title == "Bonjour").unwrap();
    assert_eq!(
        (bonjour.text.as_str(), bonjour.answer.trim_end()),
        (
            "Hello, \"world\"",
            "Bonjour
le monde"
        )
    );
    assert_eq!(bonjour.tags, ["greeting", "a,b"]);
    assert_eq!(
        from_csv("front\tback\nq\ta\n").unwrap().items.len(),
        1,
        "tab separated"
    );

    let zip = read_zip(&export_markdown_zip(&s, Some("french")).unwrap()).unwrap();
    let names: Vec<&str> = zip.iter().map(|(p, _)| p.as_str()).collect();
    assert!(
        names.contains(&"french/a.md")
            && names.contains(&"french/verbs/_stack.md")
            && names.contains(&"french/pic.png")
    );
    assert!(!names.iter().any(|n| n.starts_with("maths")));
}
