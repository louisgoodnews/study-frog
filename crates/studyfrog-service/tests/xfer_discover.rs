#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Finding Obsidian vaults, Anki packages and Anki profiles to suggest for import.

use std::fs;
use std::path::PathBuf;

use studyfrog_service::xfer_discover::{FoundKind, discover, places};

fn home(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("studyfrog-disc-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn nothing_installed_finds_nothing_and_does_not_fail() {
    let h = home("none");
    assert!(discover(&places(&h, None, None)).is_empty());
    let _ = fs::remove_dir_all(&h);
}

#[test]
fn obsidian_vaults_packages_and_anki_profiles_are_found_with_counts() {
    let h = home("all");
    let vault = h.join("Notes/Psych");
    fs::create_dir_all(vault.join("sub")).unwrap();
    fs::create_dir_all(vault.join(".obsidian")).unwrap();
    for f in ["a.md", "b.MD", "sub/c.md", ".obsidian/hidden.md", "pic.png"] {
        fs::write(vault.join(f), "x").unwrap();
    }
    fs::create_dir_all(h.join(".config/obsidian")).unwrap();
    let missing = h.join("gone");
    fs::write(
        h.join(".config/obsidian/obsidian.json"),
        format!(
            r#"{{"vaults":{{"1":{{"path":"{}","ts":1}},"2":{{"path":"{}"}},"3":{{"path":"relative/x"}}}}}}"#,
            vault.display(),
            missing.display()
        ),
    )
    .unwrap();
    fs::create_dir_all(h.join("Downloads")).unwrap();
    fs::write(h.join("Downloads/Deck.apkg"), vec![0u8; 2048]).unwrap();
    fs::write(h.join("Downloads/notes.txt"), "x").unwrap();
    fs::create_dir_all(h.join(".local/share/Anki2/User 1")).unwrap();
    fs::write(h.join(".local/share/Anki2/User 1/collection.anki2"), "x").unwrap();
    fs::create_dir_all(h.join(".local/share/Anki2/addons21")).unwrap();

    let found = discover(&places(&h, None, None));
    let kinds: Vec<_> = found.iter().map(|f| f.kind).collect();
    assert_eq!(
        kinds,
        [
            FoundKind::ObsidianVault,
            FoundKind::AnkiPackage,
            FoundKind::AnkiProfile
        ]
    );
    assert_eq!(found[0].name, "Psych");
    assert_eq!(
        found[0].detail, "3 Markdown files",
        "hidden folders and other files are not counted"
    );
    assert_eq!(found[1].name, "Deck.apkg");
    assert!(found[1].detail.starts_with("2 KB"), "{}", found[1].detail);
    assert_eq!(found[2].name, "User 1");
    let _ = fs::remove_dir_all(&h);
}

#[test]
fn xdg_variables_are_used_when_absolute_and_ignored_otherwise() {
    let h = home("xdg");
    let p = places(&h, Some("/data"), Some("relative"));
    assert_eq!(p.data, PathBuf::from("/data"));
    assert_eq!(p.config, h.join(".config"));
    let _ = fs::remove_dir_all(&h);
}
