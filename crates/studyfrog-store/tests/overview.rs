#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The UI-facing overview and the per-user app config.

mod common;

use common::vault::{card, note, stack, write};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::settings::{AppSettings, with_opened_vault};
use studyfrog_core::uid::Uid;
use studyfrog_store::app_config::{app_config_dir, read_app_settings, write_app_settings};
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::overview::load_overview;
use studyfrog_store::reindex::reindex;
use studyfrog_store::schema::MIGRATIONS;

#[test]
fn overview_lists_stacks_counts_inbox_and_problems() {
    let dir = common::temp_dir("overview");
    let r = &dir.path;
    write(r, "french/_stack.md", &stack(1, "French"));
    write(
        r,
        "french/_stack.toml",
        "scheduler = \"sm2\"\ndaily_new_limit = 5\n",
    );
    write(r, "french/a.md", &card(10, "A", "q", "a"));
    write(r, "french/b.md", &card(11, "B", "q", "a"));
    write(r, "french/c.md", &note(12, "C", ""));
    write(r, "maths/_stack.md", &stack(2, "Maths"));
    write(r, "maths/_stack.toml", "daily_new_limit = oops\n");
    write(r, "loose.md", &note(13, "Loose", ""));
    write(r, "broken.md", "no front matter");
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    reindex(&mut conn, r).unwrap();

    let o = load_overview(&conn).unwrap();
    let rows: Vec<_> = o
        .stacks
        .iter()
        .map(|s| {
            (
                s.uid,
                s.title.as_str(),
                s.folder.as_str(),
                s.notes,
                s.cards,
                s.questions,
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (Uid(1), "French", "french", 1, 2, 0),
            (Uid(2), "Maths", "maths", 0, 0, 0)
        ]
    );
    assert_eq!(
        (o.stacks[0].scheduler, o.stacks[0].daily_new_limit),
        (Some(Scheduler::Sm2), Some(5))
    );
    assert_eq!(o.inbox_items, 1);
    let problems: Vec<_> = o.problems.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(
        problems,
        ["broken.md", "maths"],
        "a bad file and a bad _stack.toml"
    );
}

#[test]
fn app_config_dir_follows_xdg_rules() {
    let d = |x, h| app_config_dir(x, h).map(|p| p.display().to_string());
    assert_eq!(
        d(Some("/x"), Some("/home/u")).as_deref(),
        Some("/x/studyfrog")
    );
    assert_eq!(
        d(None, Some("/home/u")).as_deref(),
        Some("/home/u/.config/studyfrog")
    );
    assert_eq!(
        d(Some(""), Some("/home/u")).as_deref(),
        Some("/home/u/.config/studyfrog"),
        "empty counts as unset"
    );
    assert_eq!(d(None, None), None);
}

#[test]
fn app_settings_persist_and_a_missing_file_means_defaults() {
    let dir = common::temp_dir("appcfg");
    let cfg = dir.path.join("nested/studyfrog");
    assert_eq!(read_app_settings(&cfg).unwrap(), AppSettings::default());
    let s = with_opened_vault(
        &with_opened_vault(&AppSettings::default(), "/a"),
        "/b c/\"d\"",
    );
    write_app_settings(&cfg, &s).unwrap();
    assert_eq!(read_app_settings(&cfg).unwrap(), s);
    assert!(
        !cfg.join("app.toml.tmp").exists(),
        "the temporary file is gone"
    );
    std::fs::write(cfg.join("app.toml"), "vault = 5\n").unwrap();
    assert!(read_app_settings(&cfg).is_err());
}
