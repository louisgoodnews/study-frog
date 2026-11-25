// Global state (level and ring) is changed here, so all checks that change it run in one test
// function. Records use unique targets so that other tests in this binary cannot disturb them.
use studyfrog_log::{Level, RING_CAPACITY, emit, level, recent, set_level};

fn with_target(target: &str) -> Vec<studyfrog_log::Record> {
    recent(usize::MAX)
        .into_iter()
        .filter(|record| record.target == target)
        .collect()
}

#[test]
fn global_level_ring_and_macros_behave() {
    // Records below the minimum are dropped; records at or above it are kept.
    assert_eq!(level(), Level::Info);
    set_level(Level::Warn);
    emit(Level::Info, "dispatch-drop", "dropped", &[]);
    emit(Level::Warn, "dispatch-drop", "kept", &[]);
    let kept = with_target("dispatch-drop");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].message, "kept");
    set_level(Level::Info);

    // The macros format the message and the fields, and forward them to emit.
    studyfrog_log::log_info!("dispatch-macro", "opened {}", "stack"; count = 2, folder = "A B");
    studyfrog_log::log_debug!("dispatch-macro", "not shown at Info");
    let macro_records = with_target("dispatch-macro");
    assert_eq!(macro_records.len(), 1);
    assert_eq!(macro_records[0].message, "opened stack");
    assert_eq!(
        macro_records[0].fields,
        vec![
            ("count".to_string(), "2".to_string()),
            ("folder".to_string(), "A B".to_string()),
        ]
    );

    // The ring keeps at most RING_CAPACITY records, and recent(n) returns at most n.
    assert_eq!(RING_CAPACITY, 512);
    for i in 0..600 {
        emit(
            Level::Error,
            "dispatch-ring",
            "tick",
            &[("i", i.to_string().as_str())],
        );
    }
    assert!(recent(usize::MAX).len() <= RING_CAPACITY);
    assert!(recent(3).len() <= 3);
    assert!(recent(0).is_empty());
}
