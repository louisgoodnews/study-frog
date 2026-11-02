#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The pure update function: every transition, no I/O.

use studyfrog_core::settings::{AppSettings, vault_settings_default};
use studyfrog_store::overview::VaultOverview;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen, State, VaultInfo};
use studyfrog_ui::update::update;

fn info(root: &str) -> VaultInfo {
    VaultInfo {
        root: root.to_owned(),
        name: root.rsplit('/').next().unwrap().to_owned(),
        overview: VaultOverview::default(),
        settings: vault_settings_default(),
        settings_error: None,
    }
}

fn run(state: State, msgs: Vec<Msg>) -> (State, Vec<Effect>) {
    msgs.into_iter()
        .fold((state, Vec::new()), |(s, mut fx), m| {
            let (s, more) = update(s, m);
            fx.extend(more);
            (s, fx)
        })
}

fn open_vault(path: &str) -> State {
    update(State::default(), Msg::VaultOpened(Ok(info(path)))).0
}

#[test]
fn starting_with_a_remembered_vault_opens_it() {
    let app = AppSettings {
        vault: Some("/v/notes".into()),
        recent: vec!["/v/notes".into()],
        reading_wpm: None,
        answer_engine: Default::default(),
        reading: Default::default(),
        settings_tab: None,
        views_untracked: false,
        library_columns: None,
        calendar_reviews_off: false,
        stack_tabs: Vec::new(),
    };
    let (s, fx) = update(State::default(), Msg::Started { app, error: None });
    assert_eq!((s.path_input.as_str(), s.opening), ("/v/notes", true));
    assert_eq!(fx, [Effect::OpenVault("/v/notes".into())]);
    assert_eq!(
        update(
            State::default(),
            Msg::Started {
                app: AppSettings::default(),
                error: None
            }
        )
        .1,
        []
    );
}

#[test]
fn unreadable_app_settings_are_reported_but_not_fatal() {
    let (s, fx) = update(
        State::default(),
        Msg::Started {
            app: AppSettings::default(),
            error: Some("bad".into()),
        },
    );
    assert!(fx.is_empty() && s.screen == Screen::Welcome);
    let n = s.notice.unwrap();
    assert_eq!(n.kind, NoticeKind::Error);
    assert!(n.text.contains("bad"));
}

#[test]
fn opening_needs_a_path_and_ignores_double_clicks() {
    let (s, fx) = run(
        State::default(),
        vec![Msg::PathEdited("   ".into()), Msg::OpenVaultClicked],
    );
    assert!(fx.is_empty() && !s.opening);
    assert_eq!(s.notice.unwrap().kind, NoticeKind::Error);

    let (s, fx) = run(
        State::default(),
        vec![
            Msg::PathEdited(" /v/a ".into()),
            Msg::OpenVaultClicked,
            Msg::OpenVaultClicked,
        ],
    );
    assert!(s.opening);
    assert_eq!(
        fx,
        [Effect::OpenVault("/v/a".into())],
        "the second click is ignored while opening"
    );
}

#[test]
fn a_successful_open_shows_the_library_remembers_the_vault_and_saves_it() {
    let (s, fx) = update(State::default(), Msg::VaultOpened(Ok(info("/v/a"))));
    assert_eq!((s.screen, s.opening), (Screen::Home, false));
    assert_eq!(s.app.vault.as_deref(), Some("/v/a"));
    assert!(s.notice.is_none(), "opening a vault is not announced");
    assert_eq!(fx.len(), 4);
    assert!(
        matches!(fx.last(), Some(Effect::Notifications(_))),
        "the vault's stored notifications are asked for"
    );
    assert!(matches!(&fx[0], Effect::SaveAppSettings(a) if a.recent == ["/v/a"]));
    assert_eq!(
        fx[1],
        Effect::LoadHome,
        "the home screen asks what is ready"
    );
}

#[test]
fn a_failed_open_returns_to_the_welcome_screen_with_the_reason() {
    let (s, _) = run(
        State {
            opening: true,
            ..State::default()
        },
        vec![Msg::VaultOpened(Err("no such folder".into()))],
    );
    assert!(!s.opening && s.vault.is_none() && s.screen == Screen::Welcome);
    assert!(s.notice.unwrap().text.contains("no such folder"));
}

#[test]
fn navigation_needs_an_open_vault_and_closing_resets() {
    let (s, _) = update(State::default(), Msg::Navigate(Screen::Settings));
    assert_eq!(s.screen, Screen::Welcome, "no vault, no navigation");
    let s = open_vault("/v/a");
    let (s, _) = update(s, Msg::Navigate(Screen::Rehearse));
    assert_eq!(s.screen, Screen::Rehearse);
    let (s, _) = update(s, Msg::CloseVault);
    assert!(s.vault.is_none() && s.screen == Screen::Welcome);
    assert!(!s.polling);
}

#[test]
fn ticks_poll_once_at_a_time_and_only_with_a_vault() {
    assert_eq!(update(State::default(), Msg::Tick).1, []);
    let s = open_vault("/v/a");
    let (s, fx) = update(s, Msg::Tick);
    assert_eq!(fx, [Effect::PollVault, Effect::CheckReminders]);
    assert!(s.polling);
    assert_eq!(
        update(s.clone(), Msg::Tick).1,
        [],
        "no second poll while one is running"
    );

    let (s2, _) = update(s.clone(), Msg::VaultPolled(Ok(None)));
    assert!(!s2.polling && s2.vault == s.vault);
    let mut newer = info("/v/a");
    newer.name = "renamed".into();
    let (s3, _) = update(s, Msg::VaultPolled(Ok(Some(newer))));
    assert_eq!(s3.vault.unwrap().name, "renamed");
    assert!(!s3.polling);
}

#[test]
fn a_poll_result_after_closing_the_vault_is_dropped_and_errors_are_shown() {
    let (s, _) = update(State::default(), Msg::VaultPolled(Ok(Some(info("/v/a")))));
    assert!(s.vault.is_none());
    let (s, _) = update(
        open_vault("/v/a"),
        Msg::VaultPolled(Err("disk gone".into())),
    );
    assert!(s.notice.unwrap().text.contains("disk gone"));
}

#[test]
fn notices_can_be_dismissed_and_save_failures_are_reported() {
    let (s, _) = update(
        open_vault("/v/a"),
        Msg::AppSettingsSaved(Err("read-only".into())),
    );
    assert!(s.notice.as_ref().unwrap().text.contains("read-only"));
    let (s, _) = update(s, Msg::DismissNotice);
    assert!(s.notice.is_none());
    assert!(
        update(State::default(), Msg::AppSettingsSaved(Ok(())))
            .0
            .notice
            .is_none()
    );
}
