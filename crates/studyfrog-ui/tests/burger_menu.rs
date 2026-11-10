#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for floating burger menu research prototype.

use studyfrog_core::settings::vault_settings_default;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, State, VaultInfo};
use studyfrog_ui::update::update;

#[test]
fn burger_menu_screens_navigate() {
    let mut state = State::default();
    assert_eq!(state.screen, Screen::Welcome);

    let (state_help, _) = update(state.clone(), Msg::Navigate(Screen::Help));
    assert_eq!(state_help.screen, Screen::Help);

    // When a vault is open, all screens navigate
    state.vault = Some(VaultInfo {
        name: "Test Vault".to_owned(),
        root: "/mock".into(),
        overview: Default::default(),
        settings: vault_settings_default(),
        settings_error: None,
    });

    let (state_cal, _) = update(state.clone(), Msg::Navigate(Screen::Calendar));
    assert_eq!(state_cal.screen, Screen::Calendar);

    let (state_console, _) = update(state.clone(), Msg::Navigate(Screen::Console));
    assert_eq!(state_console.screen, Screen::Console);
}
