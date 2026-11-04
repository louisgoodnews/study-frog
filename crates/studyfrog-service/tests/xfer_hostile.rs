#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Malformed and hostile import files: errors in the preview, never a crash.

use std::fs;

use studyfrog_service::create::IdSource;
use studyfrog_service::session::open_vault;
use studyfrog_service::xfer_csv::from_csv;
use studyfrog_service::xfer_import::{Policy, apply_import, counts, plan_import};
use studyfrog_service::xfer_json::from_json;
use studyfrog_service::xfer_zip::{read_zip, write_zip};

fn ids() -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000,
        random: [7; 10],
    }
}

#[test]
fn malformed_and_hostile_files_are_errors_in_the_preview_and_never_panic() {
    let json_cases = [
        "",
        "{",
        "[]",
        "null",
        "{\"format\":\"x\"}",
        "{\"format\":\"studyfrog-export\",\"version\":99}",
        "{\"format\":\"studyfrog-export\",\"version\":1,\"items\":[{\"kind\":\"zzz\"}]}",
        "{\"format\":\"studyfrog-export\",\"version\":1,\"items\":[{\"kind\":\"card\",\"id\":\"nope\"}]}",
    ];
    for c in json_cases {
        assert!(from_json(c).is_err(), "{c:?}");
    }
    for c in [
        "",
        "\"unterminated,front,back\n",
        "a,b\n1,2\n",
        "\u{0}\u{0}",
        &"x,".repeat(100_000),
    ] {
        let _ = from_csv(c);
    }
    for c in [&b""[..], b"PK\x03\x04garbage", &[0xff; 64], b"not a zip"] {
        assert!(read_zip(c).is_err(), "{c:?}");
    }
    let hostile = write_zip(&[
        ("../evil.txt".to_owned(), vec![1]),
        ("/abs.txt".to_owned(), vec![2]),
        ("ok.txt".to_owned(), vec![3]),
    ])
    .unwrap();
    let names: Vec<String> = read_zip(&hostile)
        .unwrap()
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    assert_eq!(
        names,
        ["abs.txt", "ok.txt"],
        "a path that leaves the folder is dropped; a leading slash is made relative"
    );

    let dir = std::env::temp_dir().join(format!("studyfrog-hostile-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let (mut t, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let d = Dir(dir);
    let mut b = from_json("{\"format\":\"studyfrog-export\",\"version\":1,\"items\":[{\"kind\":\"card\",\"stack\":\"../../etc\",\"text\":\"q\",\"answer\":\"a\"},{\"kind\":\"card\",\"stack\":\"/abs\",\"text\":\"q\",\"answer\":\"a\"},{\"kind\":\"card\",\"stack\":\".hidden\",\"text\":\"q\",\"answer\":\"a\"},{\"kind\":\"card\",\"stack\":\"ok\",\"text\":\"q\",\"answer\":\"a\"}]}").unwrap();
    b.files = vec![
        ("../escape.png".to_owned(), vec![1]),
        ("fine/pic.png".to_owned(), vec![2]),
    ];
    let plan = plan_import(&t, &b, None).unwrap();
    assert_eq!(counts(&plan), (1, 0, 3));
    apply_import(&mut t, &b, None, (Policy::Skip, false), ids()).unwrap();
    assert!(
        !d.0.parent().unwrap().join("escape.png").exists() && d.0.join("fine/pic.png").is_file()
    );
    assert!(!d.0.join("etc").exists());
}

struct Dir(std::path::PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_adapter_table_picks_a_reader_by_extension_and_refuses_the_unknown() {
    use studyfrog_service::xfer_adapters::{ADAPTERS, adapter_for, read_source};
    assert_eq!(adapter_for("a.APKG").unwrap().name, "Anki package");
    assert_eq!(adapter_for("x.tsv").unwrap().name, "CSV or TSV cards");
    assert!(adapter_for("x.docx").is_none() && adapter_for("noextension").is_none());
    assert_eq!(
        adapter_for("test.toml").unwrap().name,
        "StudyFrog export (TOML)"
    );
    assert_eq!(
        adapter_for("test.ron").unwrap().name,
        "StudyFrog export (RON)"
    );
    assert_eq!(ADAPTERS.len(), 6);
    let dir = std::env::temp_dir().join(format!("studyfrog-adapters-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("cards.csv"), "front,back\nq,a\n").unwrap();
    fs::write(dir.join("odd.docx"), "x").unwrap();
    fs::write(dir.join("bad.json"), "{").unwrap();
    let (b, name) = read_source(&dir.join("cards.csv")).unwrap();
    assert_eq!((b.items.len(), name), (1, "CSV or TSV cards"));
    assert!(
        read_source(&dir.join("odd.docx"))
            .unwrap_err()
            .contains("no importer")
    );
    assert!(read_source(&dir.join("bad.json")).is_err());
    assert!(read_source(&dir.join("missing.csv")).is_err());
    let _ = fs::remove_dir_all(&dir);
}
