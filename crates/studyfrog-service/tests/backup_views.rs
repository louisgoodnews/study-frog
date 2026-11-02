#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Recorded item views are part of a backup and come back with a restore.

use std::fs;

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_core::views::{ViewEvent, ViewKind};
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::backup::{BackupOptions, create_backup};
use studyfrog_service::restore::restore_backup;
use studyfrog_service::session::open_vault;
use studyfrog_store::views::{insert_view, load_views};

#[test]
fn views_survive_a_backup_and_a_restore() {
    let base = std::env::temp_dir().join(format!("studyfrog-backup-views-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let vault = base.join("vault");
    fs::create_dir_all(vault.join("deck")).unwrap();
    let card = format!(
        "---\nid: {}\nkind: card\ntitle: C\n---\nq\n???\na\n",
        format_ulid(Uid(2))
    );
    fs::write(vault.join("deck/c.md"), card).unwrap();
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();
    let seen = ViewEvent {
        item: Uid(2),
        kind: ViewKind::View,
        at: UnixMs(1_700_000_000_000),
        active_ms: 42_000,
    };
    insert_view(&session.conn, &seen).unwrap();

    let options = BackupOptions {
        config_dir: None,
        include_keys: false,
        now_ms: 1_800_000_000_000,
        app_version: "0.3.0",
    };
    let report = create_backup(&session, &base.join("backups"), options).unwrap();
    let target = base.join("restored");
    restore_backup(&report.path, &target, None).unwrap();
    let (restored, _) = open_vault(target.to_str().unwrap()).unwrap();
    assert_eq!(load_views(&restored.conn, Uid(2)).unwrap(), [seen]);
    let _ = fs::remove_dir_all(&base);
}
