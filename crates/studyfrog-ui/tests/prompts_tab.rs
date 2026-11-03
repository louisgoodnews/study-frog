#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0206: the Prompts tab lists the prompts, saves one or all of them, and the import dialog points to it.

mod common;

use std::path::PathBuf;
use std::sync::Mutex;

use common::views::{assert_all_named, harness, open_model};
use common::{run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::agent_types::{AgentMsg, AgentTab};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::prompts_ui::{PromptsEffect, PromptsMsg};
use studyfrog_ui::state::Screen;

static FOLDER: Mutex<Option<PathBuf>> = Mutex::new(None);

fn pick() -> Option<PathBuf> {
    FOLDER.lock().unwrap().clone()
}

fn on_the_tab() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Agent(AgentMsg::Tab(AgentTab::Prompts)));
    apply(&mut m, Msg::Navigate(Screen::Agent));
    m
}

#[test]
fn the_tab_lists_the_six_prompts_with_copy_and_save() {
    let mut h = harness(on_the_tab());
    h.run_steps(6);
    for title in [
        "Create flashcards from a file",
        "Analyze and critique a file",
    ] {
        assert!(h.query_by_label(title).is_some(), "{title}");
    }
    assert_eq!(h.query_all_by_label("Copy to clipboard").count(), 6);
    assert_eq!(h.query_all_by_label("Save as .md").count(), 6);
    assert_all_named(&h, "prompts tab");
    h.get_all_by_label("Save as .md").next().unwrap().click();
    h.run_steps(3);
    assert!(
        h.state()
            .1
            .contains(&Effect::Prompts(PromptsEffect::Export(vec!["flashcards"])))
    );
    h.get_by_label("Save all prompts to a folder…").click();
    h.run_steps(3);
    let all = h
        .state()
        .1
        .iter()
        .any(|e| matches!(e, Effect::Prompts(PromptsEffect::Export(ids)) if ids.len() == 6));
    assert!(all);
}

#[test]
fn saving_writes_one_markdown_file_per_prompt_into_the_chosen_folder_or_nothing_when_cancelled() {
    let dir = temp_dir("prompts");
    let mut w = worker_new(None);
    w.folder_picker = Some(pick);
    let mut m = open_model();
    *FOLDER.lock().unwrap() = None;
    let fx = apply(&mut m, Msg::Prompts(PromptsMsg::SaveAll));
    run_to_idle(&mut m, &mut w, fx);
    assert!(
        std::fs::read_dir(&dir.path).unwrap().next().is_none(),
        "cancelled: nothing written"
    );

    *FOLDER.lock().unwrap() = Some(dir.path.clone());
    let fx = apply(&mut m, Msg::Prompts(PromptsMsg::SaveAll));
    run_to_idle(&mut m, &mut w, fx);
    let names: std::collections::BTreeSet<String> = std::fs::read_dir(&dir.path)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names.len(), 6);
    assert!(names.contains("studyfrog-prompt-flashcards.md"));
    let text = std::fs::read_to_string(dir.path.join("studyfrog-prompt-stack.md")).unwrap();
    assert!(
        text.starts_with("# Create a whole stack from a file")
            && text.contains("stack,kind,title,front,back,tags")
    );
    assert!(
        m.state
            .notice
            .as_ref()
            .unwrap()
            .text
            .starts_with("Saved 6 prompts")
    );
}
