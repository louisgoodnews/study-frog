#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for ScriptStudio multi-tab environment, FrogScript pipeline execution, and console streams.

use studyfrog_core::plan::ItemKind;
use studyfrog_core::settings::vault_settings_default;
use studyfrog_core::uid::Uid;
use studyfrog_store::overview::ItemRow;
use studyfrog_ui::console_ui::{ConsoleMsg, EntryKind};
use studyfrog_ui::script_studio::{
    ConsoleFilter, active_tab, close_tab, new_tab, select_tab, tab_new,
};
use studyfrog_ui::script_studio_exec::{apply_studio_msg, execute_studio_script};
use studyfrog_ui::state::State;

fn sample_state() -> State {
    State {
        browser: studyfrog_ui::state::Browser {
            items: vec![
                ItemRow {
                    uid: Uid(1),
                    title: "Synapse Plasticity".into(),
                    kind: ItemKind::Card,
                    path: "Neuroscience/Synapses/plasticity.md".into(),
                    due: None,
                    status: studyfrog_core::status::Status::Learning,
                },
                ItemRow {
                    uid: Uid(2),
                    title: "Action Potential".into(),
                    kind: ItemKind::Card,
                    path: "Neuroscience/Synapses/action_potential.md".into(),
                    due: None,
                    status: studyfrog_core::status::Status::Reviewing,
                },
                ItemRow {
                    uid: Uid(3),
                    title: "Linear Algebra Notes".into(),
                    kind: ItemKind::Note,
                    path: "Math/algebra.md".into(),
                    due: None,
                    status: studyfrog_core::status::Status::New,
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn script_studio_multi_tab_lifecycle() {
    let mut state = sample_state();
    assert_eq!(state.console.studio.tabs.len(), 1);
    assert_eq!(
        active_tab(&state.console.studio).unwrap().name,
        "quickstart.frog"
    );

    // Add a new tab
    new_tab(&mut state.console.studio, Some("review_boost.frog"));
    assert_eq!(state.console.studio.tabs.len(), 2);
    assert_eq!(state.console.studio.active_tab, 1);
    assert_eq!(
        active_tab(&state.console.studio).unwrap().name,
        "review_boost.frog"
    );

    // Select first tab
    select_tab(&mut state.console.studio, 0);
    assert_eq!(state.console.studio.active_tab, 0);

    // Close second tab
    close_tab(&mut state.console.studio, 1);
    assert_eq!(state.console.studio.tabs.len(), 1);
    assert_eq!(state.console.studio.active_tab, 0);
}

#[test]
fn frogscript_execution_in_studio_emits_logs_and_event() {
    let mut state = sample_state();
    state.console.studio.tabs[0] = tab_new(
        "test.frog",
        r#"
cards = from stack "Neuroscience/Synapses" | where type "card"
log_info("Found " + len(cards) + " synapse cards")
log_warn("Warning test")
cards | set priority "high"
"#,
        None,
    );

    let (res, logs) = execute_studio_script(&state.console.studio, &state);
    assert!(res.is_ok(), "execution should succeed: {res:?}");
    assert_eq!(logs.len(), 2);
    assert!(logs[0].message.contains("Found 2 synapse cards"));
    assert!(logs[1].message.contains("Warning test"));

    // Reducer dispatch applies entries to console
    apply_studio_msg(&mut state, ConsoleMsg::StudioRunScript);
    let kinds: Vec<EntryKind> = state.console.entries.iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&EntryKind::Stdout));
    assert!(kinds.contains(&EntryKind::Script));
    assert!(kinds.contains(&EntryKind::Event));
}

#[test]
fn frogscript_closure_and_pipeline_execution() {
    let mut state = sample_state();
    state.console.studio.tabs[0] = tab_new(
        "closure.frog",
        r#"
items = from stack * | where type "card" | {
    card -> if card.difficulty == "normal": card | set priority "urgent"
}
log_info("Processed cards: " + len(items))
"#,
        None,
    );

    let (res, logs) = execute_studio_script(&state.console.studio, &state);
    assert!(res.is_ok(), "closure script should succeed: {res:?}");
    assert_eq!(logs.len(), 1);
    assert!(logs[0].message.contains("Processed cards: 2"));
}

#[test]
fn output_stream_filter_selection() {
    let mut state = sample_state();
    assert_eq!(state.console.studio.filter, ConsoleFilter::All);

    apply_studio_msg(
        &mut state,
        ConsoleMsg::StudioSetFilter(ConsoleFilter::Stderr),
    );
    assert_eq!(state.console.studio.filter, ConsoleFilter::Stderr);

    apply_studio_msg(
        &mut state,
        ConsoleMsg::StudioSetFilter(ConsoleFilter::Stdout),
    );
    assert_eq!(state.console.studio.filter, ConsoleFilter::Stdout);
}

#[test]
fn script_saving_to_vault_scripts_directory() {
    let temp_vault =
        std::env::temp_dir().join(format!("studyfrog_test_vault_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_vault);
    std::fs::create_dir_all(&temp_vault).unwrap();

    let mut state = sample_state();
    state.vault = Some(studyfrog_service::session::VaultInfo {
        root: temp_vault.display().to_string(),
        name: "TestVault".into(),
        overview: Default::default(),
        settings: vault_settings_default(),
        settings_error: None,
    });

    state.console.studio.tabs[0] = tab_new("cram.frog", "log_info(\"hello vault\")", None);
    apply_studio_msg(&mut state, ConsoleMsg::StudioSaveScript(None));

    let expected_file = temp_vault
        .join(".studyfrog")
        .join("scripts")
        .join("cram.frog");
    assert!(
        expected_file.is_file(),
        "expected script file at {}",
        expected_file.display()
    );
    assert_eq!(
        std::fs::read_to_string(&expected_file).unwrap(),
        "log_info(\"hello vault\")"
    );

    let _ = std::fs::remove_dir_all(&temp_vault);
}
