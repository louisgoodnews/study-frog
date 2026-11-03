#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! User profiles: the menu, the update logic and the worker with real files.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_config::profiles::Profiles;
use studyfrog_core::settings::AppSettings;
use studyfrog_ui::chrome::{ChromeMsg, Menu};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::profile_ui::ProfileMsg;
use studyfrog_ui::state::Screen;

fn two() -> Profiles {
    Profiles {
        current: "Default".into(),
        names: vec!["Default".into(), "School".into()],
    }
}

fn model() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Profile(ProfileMsg::Loaded(two())));
    m
}

#[test]
fn creating_checks_the_name_and_switching_closes_the_vault_and_opens_the_profiles_own() {
    let mut m = model();
    apply(&mut m, Msg::Profile(ProfileMsg::Draft("school".into())));
    assert!(apply(&mut m, Msg::Profile(ProfileMsg::Create)).is_empty());
    assert!(m.state.notice.as_ref().unwrap().text.contains("already"));
    apply(&mut m, Msg::Profile(ProfileMsg::Draft("Work".into())));
    assert_eq!(
        apply(&mut m, Msg::Profile(ProfileMsg::Create)),
        [Effect::CreateProfile("Work".into())]
    );
    assert!(
        apply(&mut m, Msg::Profile(ProfileMsg::Switch("Default".into()))).is_empty(),
        "already in use"
    );
    assert_eq!(
        apply(&mut m, Msg::Profile(ProfileMsg::Switch("School".into()))),
        [Effect::SwitchProfile("School".into())]
    );

    let school = Profiles {
        current: "School".into(),
        ..two()
    };
    let app = AppSettings {
        vault: Some("/v/school".into()),
        ..AppSettings::default()
    };
    let fx = apply(
        &mut m,
        Msg::Profile(ProfileMsg::Changed(Ok(Box::new((school, app))))),
    );
    assert_eq!(m.state.profiles.list.current, "School");
    assert!(
        m.state.vault.is_none() && m.state.screen == Screen::Welcome
            || fx.contains(&Effect::OpenVault("/v/school".into()))
    );
    assert!(
        fx.contains(&Effect::OpenVault("/v/school".into())),
        "{fx:?}"
    );
}

#[test]
fn deleting_needs_a_second_click_and_failures_are_reported() {
    let mut m = model();
    assert!(apply(&mut m, Msg::Profile(ProfileMsg::ConfirmDelete)).is_empty());
    apply(&mut m, Msg::Profile(ProfileMsg::AskDelete("School".into())));
    apply(&mut m, Msg::Profile(ProfileMsg::CancelDelete));
    assert!(
        apply(&mut m, Msg::Profile(ProfileMsg::ConfirmDelete)).is_empty(),
        "cancelled"
    );
    apply(&mut m, Msg::Profile(ProfileMsg::AskDelete("School".into())));
    assert_eq!(
        apply(&mut m, Msg::Profile(ProfileMsg::ConfirmDelete)),
        [Effect::DeleteProfile("School".into())]
    );
    apply(&mut m, Msg::Profile(ProfileMsg::Changed(Err("no".into()))));
    assert!(m.state.notice.as_ref().unwrap().text.contains("no"));
}

#[test]
fn the_profile_menu_lists_profiles_and_has_no_vault_path() {
    let mut m = model();
    apply(&mut m, Msg::Chrome(ChromeMsg::ToggleMenu(Menu::Profile)));
    let mut h = harness(m);
    h.run_steps(2);
    for text in [
        "Profile: Default",
        "Switch to School",
        "Delete School",
        "New profile",
        "Create profile",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert!(
        h.query_by_label("/v/notes").is_none(),
        "the vault path is not shown"
    );
    h.get_by_label("Delete School").click();
    h.run_steps(2);
    assert!(h.query_by_label("Really delete School").is_some());
    h.get_by_label("Switch to School").click();
    h.run_steps(2);
    assert!(
        h.state()
            .1
            .contains(&Effect::SwitchProfile("School".into()))
    );
    assert_all_named(&h, "profile menu");
}

fn root(label: &str) -> std::path::PathBuf {
    let root =
        std::env::temp_dir().join(format!("studyfrog-profiles-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("app.toml"), "vault = \"/v/default\"\n").unwrap();
    root
}

#[test]
fn the_worker_starts_with_the_profile_in_use_and_gives_a_new_one_its_own_folder() {
    let root = root("make");
    let mut w = worker_new(Some(root.clone()));
    let start = execute(&mut w, Effect::LoadAppSettings, 0);
    assert!(matches!(&start[0], Msg::Profile(ProfileMsg::Loaded(p)) if p.current == "Default"));
    assert!(
        matches!(&start[1], Msg::Started { app, .. } if app.vault.as_deref() == Some("/v/default"))
    );
    let made = execute(&mut w, Effect::CreateProfile("Exam prep".into()), 0);
    let Msg::Profile(ProfileMsg::Changed(Ok(both))) = &made[0] else {
        panic!("{:?}", made[0])
    };
    let (p, app) = &**both;
    assert_eq!((p.current.as_str(), app.vault.clone()), ("Exam prep", None));
    assert!(
        root.join("profiles/exam-prep/app.toml").is_file() && root.join("profiles.toml").is_file()
    );
    assert_eq!(
        w.config_dir.as_deref(),
        Some(root.join("profiles/exam-prep").as_path())
    );
    let back = execute(&mut w, Effect::SwitchProfile("Default".into()), 0);
    let Msg::Profile(ProfileMsg::Changed(Ok(both))) = &back[0] else {
        panic!("{:?}", back[0])
    };
    let app = &both.1;
    assert_eq!(
        app.vault.as_deref(),
        Some("/v/default"),
        "the default profile kept its own settings"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_worker_refuses_unknown_and_protected_profiles_and_deletes_the_rest() {
    let root = root("drop");
    let mut w = worker_new(Some(root.clone()));
    execute(&mut w, Effect::LoadAppSettings, 0);
    execute(&mut w, Effect::CreateProfile("Exam prep".into()), 0);
    execute(&mut w, Effect::SwitchProfile("Default".into()), 0);
    let bad = execute(&mut w, Effect::SwitchProfile("Ghost".into()), 0);
    assert!(
        matches!(&bad[0], Msg::Profile(ProfileMsg::Changed(Err(e))) if e.contains("no profile"))
    );
    let protected = execute(&mut w, Effect::DeleteProfile("Default".into()), 0);
    assert!(matches!(
        &protected[0],
        Msg::Profile(ProfileMsg::Changed(Err(_)))
    ));
    let gone = execute(&mut w, Effect::DeleteProfile("Exam prep".into()), 0);
    assert!(matches!(&gone[0], Msg::Profile(ProfileMsg::Changed(Ok(_)))));
    assert!(!root.join("profiles/exam-prep").exists());
    let mut again = worker_new(Some(root.clone()));
    let start = execute(&mut again, Effect::LoadAppSettings, 0);
    assert!(matches!(&start[0], Msg::Profile(ProfileMsg::Loaded(p)) if p.names == ["Default"]));
    let _ = std::fs::remove_dir_all(&root);
}
