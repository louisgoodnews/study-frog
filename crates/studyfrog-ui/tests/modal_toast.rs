#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The first-launch toast and the vault dialog (update logic).

mod common;

use studyfrog_store::overview::{StackRow, VaultOverview};
use studyfrog_ui::modal::stack_labels;

use studyfrog_core::settings::AppSettings;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::{Modal, ModalMsg, ToastAction};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn modal(m: &mut Model, msg: ModalMsg) -> Vec<Effect> {
    apply(m, Msg::Modal(msg))
}

#[test]
fn the_first_launch_shows_a_toast_instead_of_asking_for_a_path_and_clicking_it_opens_the_vault_dialog()
 {
    let mut m = Model::default();
    apply(
        &mut m,
        Msg::Started {
            app: AppSettings::default(),
            error: None,
        },
    );
    assert_eq!(m.state.toasts.len(), 1);
    assert_eq!(m.state.toasts[0].action, ToastAction::OpenVaultDialog);
    assert_eq!(
        (m.state.screen, m.state.modal.clone()),
        (Screen::Welcome, None),
        "no dialog until the user clicks"
    );
    modal(&mut m, ModalMsg::ToastClicked(0));
    assert_eq!(m.state.modal, Some(Modal::Vault));
    assert!(m.state.toasts.is_empty(), "a clicked toast is used up");

    let mut dismissed = Model::default();
    apply(
        &mut dismissed,
        Msg::Started {
            app: AppSettings::default(),
            error: None,
        },
    );
    modal(&mut dismissed, ModalMsg::ToastDismissed(0));
    assert!(dismissed.state.toasts.is_empty() && dismissed.state.modal.is_none());
    assert!(
        modal(&mut dismissed, ModalMsg::ToastClicked(5)).is_empty(),
        "a stale index does nothing"
    );

    let mut returning = Model::default();
    let app = AppSettings {
        vault: Some("/v/notes".into()),
        recent: vec!["/v/notes".into()],
        ..AppSettings::default()
    };
    let fx = apply(&mut returning, Msg::Started { app, error: None });
    assert!(
        returning.state.toasts.is_empty(),
        "a known vault is simply opened"
    );
    assert_eq!(fx, [Effect::OpenVault("/v/notes".into())]);
}

#[test]
fn opening_a_vault_closes_the_dialog_and_clears_the_toasts() {
    let mut m = Model::default();
    apply(
        &mut m,
        Msg::Started {
            app: AppSettings {
                tutorial_offered: true,
                ..AppSettings::default()
            },
            error: None,
        },
    );
    modal(&mut m, ModalMsg::OpenVault);
    apply(&mut m, Msg::VaultOpened(Ok(common::views::info())));
    assert!(m.state.modal.is_none() && m.state.toasts.is_empty());
    assert_eq!(m.state.screen, Screen::Home);
}

fn stack(title: &str, folder: &str, parent: Option<&str>) -> StackRow {
    StackRow {
        parent: parent.map(str::to_owned),
        subjects: Vec::new(),
        ..common::views::stack(title, folder, 0, 0)
    }
}

#[test]
fn stacks_are_labelled_with_the_chain_of_stacks_they_sit_in() {
    let o = VaultOverview {
        stacks: vec![
            stack("French", "french", None),
            stack("Verbs", "french/verbs", Some("french")),
            stack("Irregular", "french/verbs/irregular", Some("french/verbs")),
            stack("Maths", "maths", None),
        ],
        ..VaultOverview::default()
    };
    let labels: Vec<String> = stack_labels(&o).into_iter().map(|(_, l)| l).collect();
    assert_eq!(
        labels,
        [
            "French",
            "French / Verbs",
            "French / Verbs / Irregular",
            "Maths"
        ]
    );
}
