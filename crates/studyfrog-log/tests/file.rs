use std::fs;
use std::path::PathBuf;

use studyfrog_log::{Level, Record, file_sink};

fn record(index: usize) -> Record {
    Record {
        level: Level::Info,
        target: "rotate".to_string(),
        message: format!("message number {index} with some padding text to make it longer"),
        fields: Vec::new(),
        unix_ms: 0,
    }
}

#[test]
fn the_file_rotates_by_size_and_keeps_the_given_number_of_copies() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("studyfrog-log-rotate-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let path = dir.join("app.log");

    // Each line is about 80 bytes, so with a 120-byte limit every file holds one line.
    let sink = file_sink(&path, 120, 2).expect("sink opens");
    for index in 0..10 {
        sink(&record(index));
    }

    assert!(path.exists());
    assert!(dir.join("app.log.1").exists());
    assert!(dir.join("app.log.2").exists());
    assert!(!dir.join("app.log.3").exists());
    assert!(fs::metadata(&path).expect("current file").len() <= 120);
    // The file rotates before a line that would overflow it, so the newest line is in `path`
    // and copy 1 holds the line before it.
    let current = fs::read_to_string(&path).expect("current file");
    assert!(current.contains("number 9"), "{current}");
    let previous = fs::read_to_string(dir.join("app.log.1")).expect("copy 1");
    assert!(previous.contains("number 8"), "{previous}");

    let _ = fs::remove_dir_all(&dir);
}
