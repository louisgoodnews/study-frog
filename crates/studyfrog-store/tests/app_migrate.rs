#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic

//! Moving the old config directory into the application home.

mod common;

use std::fs;

use studyfrog_store::app_migrate::migrate_legacy_config;

#[test]
fn old_files_are_copied_and_the_old_ones_stay() {
    let dir = common::temp_dir("migrate-copy");
    let (old, new) = (dir.path.join("old"), dir.path.join("new"));
    fs::create_dir_all(old.join("nested")).unwrap();
    fs::write(old.join("app.toml"), "a = 1\n").unwrap();
    fs::write(old.join("nested").join("x.toml"), "x = 2\n").unwrap();

    let copied = migrate_legacy_config(&old, &new).unwrap();

    assert_eq!(copied.len(), 2);
    assert_eq!(fs::read_to_string(new.join("app.toml")).unwrap(), "a = 1\n");
    assert_eq!(
        fs::read_to_string(new.join("nested").join("x.toml")).unwrap(),
        "x = 2\n"
    );
    assert!(old.join("app.toml").exists(), "the old file is not removed");
}

#[test]
fn a_file_that_is_already_there_is_never_overwritten() {
    let dir = common::temp_dir("migrate-keep");
    let (old, new) = (dir.path.join("old"), dir.path.join("new"));
    fs::create_dir_all(&old).unwrap();
    fs::create_dir_all(&new).unwrap();
    fs::write(old.join("app.toml"), "old = true\n").unwrap();
    fs::write(new.join("app.toml"), "new = true\n").unwrap();

    let copied = migrate_legacy_config(&old, &new).unwrap();

    assert!(copied.is_empty());
    assert_eq!(
        fs::read_to_string(new.join("app.toml")).unwrap(),
        "new = true\n"
    );
}

#[test]
fn a_missing_old_directory_is_not_an_error() {
    let dir = common::temp_dir("migrate-none");
    let copied = migrate_legacy_config(&dir.path.join("absent"), &dir.path.join("new")).unwrap();
    assert!(copied.is_empty());
}

#[cfg(unix)]
#[test]
fn the_keys_end_up_readable_by_the_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = common::temp_dir("migrate-secrets");
    let (old, new) = (dir.path.join("old"), dir.path.join("new"));
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("secrets.toml"), "key = \"k\"\n").unwrap();
    fs::set_permissions(old.join("secrets.toml"), fs::Permissions::from_mode(0o644)).unwrap();

    migrate_legacy_config(&old, &new).unwrap();

    let mode = fs::metadata(new.join("secrets.toml"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}
