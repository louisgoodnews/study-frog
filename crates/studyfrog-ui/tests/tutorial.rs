#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The first-run tutorial: offered once, each step opens its screen and is done when the user did it.
use studyfrog_ui::effect::EffectVault;
use studyfrog_ui::msg::MsgVault;

mod common;

use common::views::{info, stack};
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::help_types::HelpMsg;
use studyfrog_ui::modal::ModalMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;
use studyfrog_ui::tutorial::{FIRST_STEPS, Facts, Goal, TutorialMsg, met};

fn tutorial(m: &mut Model, t: TutorialMsg) -> Vec<Effect> {
    apply(m, Msg::Help(HelpMsg::Tutorial(t)))
}

#[test]
fn the_tour_is_offered_on_home_until_the_user_answers() {
    let mut m = Model::default();
    apply(&mut m, Msg::Vault(MsgVault::VaultOpened(Ok(info()))));
    assert!(!m.state.app.tutorial_offered);
    assert!(studyfrog_ui::tutorial::offered(&m.state), "the card shows");
    assert!(
        m.state.toasts.is_empty(),
        "no floating toast over the screen"
    );
    let fx = apply(&mut m, Msg::Modal(ModalMsg::TourDecline));
    assert!(m.state.app.tutorial_offered);
    assert!(
        fx.iter().any(
            |e| matches!(e, Effect::Vault(EffectVault::SaveAppSettings(a)) if a.tutorial_offered)
        ),
        "remembered across restarts"
    );
    apply(&mut m, Msg::Vault(MsgVault::VaultOpened(Ok(info()))));
    assert!(
        !studyfrog_ui::tutorial::offered(&m.state),
        "declined stays declined"
    );
}

#[test]
fn each_step_opens_its_screen_and_is_done_when_the_user_did_it() {
    let mut m = Model::default();
    apply(&mut m, Msg::Vault(MsgVault::VaultOpened(Ok(info()))));
    apply(&mut m, Msg::Modal(ModalMsg::TourStart));
    let t = m.state.help.tutorial.clone().unwrap();
    assert_eq!((t.step, t.done), (0, false));
    assert_eq!(m.state.screen, FIRST_STEPS[0].screen);

    let mut more = info();
    more.overview.stacks.push(stack("Latin", "latin", 0, 0));
    apply(
        &mut m,
        Msg::Vault(MsgVault::VaultPolled(Ok(Some(more.clone())))),
    );
    assert!(
        m.state.help.tutorial.as_ref().unwrap().done,
        "a stack was made"
    );

    tutorial(&mut m, TutorialMsg::Next);
    let t = m.state.help.tutorial.clone().unwrap();
    assert_eq!((t.step, t.done), (1, false));
    more.overview.stacks[2].cards = 1;
    apply(&mut m, Msg::Vault(MsgVault::VaultPolled(Ok(Some(more)))));
    assert!(
        m.state.help.tutorial.as_ref().unwrap().done,
        "a card was made"
    );

    tutorial(&mut m, TutorialMsg::Next);
    assert_eq!(m.state.screen, Screen::Rehearse);
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Home)));
    tutorial(&mut m, TutorialMsg::Show);
    assert_eq!(m.state.screen, Screen::Rehearse, "show me where");
    tutorial(&mut m, TutorialMsg::Stop);
    assert!(m.state.help.tutorial.is_none());
    tutorial(&mut m, TutorialMsg::Start);
    assert_eq!(
        m.state.help.tutorial.as_ref().unwrap().step,
        0,
        "repeatable"
    );
}

#[test]
fn ratings_and_sessions_count_only_after_the_step_began() {
    let base = Facts::default();
    let rated = |run: u128, n: usize| Facts {
        ratings: (Some(Uid(run)), n),
        ..base
    };
    assert!(!met(Goal::Rating, &base, &base));
    assert!(met(Goal::Rating, &base, &rated(1, 1)));
    assert!(!met(Goal::Rating, &rated(1, 2), &rated(1, 2)));
    assert!(met(Goal::Rating, &rated(1, 2), &rated(2, 1)), "a new run");
    let sessions = |n: Option<usize>| Facts {
        sessions: n,
        ..base
    };
    assert!(
        !met(Goal::Session, &sessions(None), &sessions(Some(3))),
        "the calendar was not read yet"
    );
    assert!(!met(Goal::Session, &sessions(Some(3)), &sessions(Some(3))));
    assert!(met(Goal::Session, &sessions(Some(3)), &sessions(Some(4))));
}

#[test]
fn the_tutorial_needs_a_vault() {
    let mut m = Model::default();
    tutorial(&mut m, TutorialMsg::Start);
    assert!(m.state.help.tutorial.is_none());
}
