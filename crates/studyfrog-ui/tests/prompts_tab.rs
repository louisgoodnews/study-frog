#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0206: the Prompts tab lists the prompts, saves one or all of them, and the import dialog points to it.

mod common;

use std::path::PathBuf;
use std::sync::Mutex;

use common::views::{assert_all_named, harness, open_model};
use common::{run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_service::prompts_meta::PromptCategory;
use studyfrog_ui::agent_types::{AgentMsg, AgentTab};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::prompts_ui::{PromptsEffect, PromptsMsg, visible_prompts};
use studyfrog_ui::state::Screen;

static FOLDER: Mutex<Option<PathBuf>> = Mutex::new(None);

fn pick() -> Option<PathBuf> {
    FOLDER.lock().unwrap().clone()
}

fn on_the_tab() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Prompts));
    m
}

fn ids(m: &Model) -> Vec<&'static str> {
    visible_prompts(&m.state.prompts_view)
        .iter()
        .map(|s| s.id)
        .collect()
}

#[test]
fn the_screen_lists_the_ten_prompts_with_copy_save_and_run() {
    let mut h = harness(on_the_tab());
    h.run_steps(6);
    for title in [
        "Create flashcards from a file",
        "Analyze and critique a file",
    ] {
        assert!(h.query_by_label(title).is_some(), "{title}");
    }
    assert_eq!(h.query_all_by_label("Copy to clipboard").count(), 10);
    assert_eq!(h.query_all_by_label("Save as .md").count(), 10);
    assert_eq!(h.query_all_by_label("Run with the assistant").count(), 10);
    assert_all_named(&h, "prompts tab");
    h.get_all_by_label("Save as .md").next().unwrap().click();
    h.run_steps(3);
    assert!(h.state().1.contains(&Effect::Prompts(PromptsEffect::Export(
        vec!["flashcards"],
        Vec::new()
    ))));
    h.get_by_label("Save all prompts to a folder…").click();
    h.run_steps(3);
    let all =
        h.state().1.iter().any(
            |e| matches!(e, Effect::Prompts(PromptsEffect::Export(ids, _)) if ids.len() == 10),
        );
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
    assert_eq!(names.len(), 10);
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
            .starts_with("Saved 10 prompts")
    );
}

#[test]
fn the_tab_offers_to_run_a_prompt_and_shows_what_would_be_sent_before_the_send_button() {
    let mut m = on_the_tab();
    let out = vec![studyfrog_agent::provider::Message {
        role: studyfrog_agent::provider::Role::User,
        content: "PROMPT TEXT and the attached file".into(),
    }];
    m.state.prompt_run.outgoing = Some(out);
    let mut h = harness(m);
    h.run_steps(6);
    assert!(
        h.query_by_label("Run a prompt with the assistant")
            .is_some()
    );
    assert!(h.query_by_label("Show what will be sent").is_some());
    assert!(h.query_by_label("Send").is_some() && h.query_by_label("Don't send").is_some());
    assert!(h.query_by_label_contains("This will be sent to").is_some());
    assert_all_named(&h, "prompt run");
}

#[test]
fn the_category_and_the_search_narrow_the_list() {
    let mut m = open_model();
    assert_eq!(ids(&m).len(), 10);
    apply(
        &mut m,
        Msg::Prompts(PromptsMsg::Category(Some(
            PromptCategory::InteractiveTutoring,
        ))),
    );
    assert_eq!(ids(&m), ["exam", "socratic"]);
    apply(
        &mut m,
        Msg::Prompts(PromptsMsg::Query("  SOCRATIC ".into())),
    );
    assert_eq!(ids(&m), ["socratic"]);
    apply(&mut m, Msg::Prompts(PromptsMsg::Category(None)));
    apply(
        &mut m,
        Msg::Prompts(PromptsMsg::Query("zzz nothing".into())),
    );
    assert!(ids(&m).is_empty());
}

#[test]
fn a_picked_option_replaces_the_one_before_and_goes_with_save() {
    let mut m = open_model();
    apply(&mut m, Msg::Prompts(PromptsMsg::Pick("depth", 1)));
    apply(&mut m, Msg::Prompts(PromptsMsg::Pick("depth", 3)));
    assert_eq!(m.state.prompts_view.picks, [("depth".to_owned(), 3)]);
    let fx = apply(&mut m, Msg::Prompts(PromptsMsg::Save("explain")));
    assert!(fx.contains(&Effect::Prompts(PromptsEffect::Export(
        vec!["explain"],
        vec![("depth".to_owned(), 3)]
    ))));
}

#[test]
fn the_assistant_tab_points_to_the_screen_and_the_screen_is_in_the_sidebar() {
    let mut m = open_model();
    apply(&mut m, Msg::Agent(AgentMsg::Tab(AgentTab::Prompts)));
    apply(&mut m, Msg::Navigate(Screen::Agent));
    let mut h = harness(m);
    h.run_steps(4);
    assert!(h.query_by_label("Open Prompts").is_some());
    assert!(
        h.query_by_label("Copy to clipboard").is_none(),
        "no second copy"
    );
    let mut h = harness(on_the_tab());
    h.run_steps(4);
    assert!(h.query_all_by_label("Prompts").count() >= 1);
}

#[test]
fn the_search_matches_the_category_name_too() {
    let mut m = on_the_tab();
    apply(&mut m, Msg::Prompts(PromptsMsg::Query("tutoring".into())));
    assert_eq!(ids(&m), ["exam", "socratic"]);
}
