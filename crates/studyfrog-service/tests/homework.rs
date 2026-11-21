#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests that homework saves are validated and mirrored to the vault file.

use studyfrog_calendar::homework::{HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_service::homework::{HOMEWORK_FILE, delete_homework_checked, save_homework_checked};
use studyfrog_store::homework_toml::homework_from_toml;

struct Dir(std::path::PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn open(label: &str) -> (Dir, studyfrog_service::session::VaultSession) {
    let dir = std::env::temp_dir().join(format!("sf-hw-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let (session, _) = studyfrog_service::session::open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), session)
}

fn sample(id: u128, title: &str) -> HomeworkItem {
    HomeworkItem {
        id: Uid(id),
        title: title.to_owned(),
        course: None,
        start: None,
        due: LocalTime {
            date: Date {
                year: 2026,
                month: 12,
                day: 2,
            },
            minute: 9 * 60,
        },
        estimate_min: 60,
        priority: HomeworkPriority::Low,
        status: HomeworkStatus::NotStarted,
        checklist: vec![],
        notes: String::new(),
    }
}

#[test]
fn invalid_items_are_refused_and_nothing_is_written() {
    let (dir, mut session) = open("invalid");
    assert!(save_homework_checked(&mut session, sample(1, "  ")).is_err());
    assert!(!dir.0.join(HOMEWORK_FILE).exists());
}

#[test]
fn saves_are_mirrored_and_deletes_are_too() {
    let (dir, mut session) = open("mirror");
    let all = save_homework_checked(&mut session, sample(1, "Essay")).unwrap();
    assert_eq!(all.len(), 1);
    let text = std::fs::read_to_string(dir.0.join(HOMEWORK_FILE)).unwrap();
    assert_eq!(homework_from_toml(&text).unwrap()[0].title, "Essay");
    let remaining = delete_homework_checked(&mut session, Uid(1)).unwrap();
    assert!(remaining.is_empty());
    let text = std::fs::read_to_string(dir.0.join(HOMEWORK_FILE)).unwrap();
    assert!(homework_from_toml(&text).unwrap().is_empty());
}
