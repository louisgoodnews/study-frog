#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The first-run tutorial: offered once, each step opens its screen and is done when the user did it.

mod common;

use common::views::{info, stack};
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::help_types::HelpMsg;
use studyfrog_ui::modal::{ModalMsg, ToastAction};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;
use studyfrog_ui::tutorial::{FIRST_STEPS, Facts, Goal, TutorialMsg, met};

fn tutorial(m: &mut Model, t: TutorialMsg) -> Vec<Effect> {
    apply(m, Msg::Help(HelpMsg::Tutorial(t)))
}

#[test]
fn the_tutorial_is_offered_once_after_the_first_vault_opens() {
    let mut m = Model::default();
    let fx = apply(&mut m, Msg::VaultOpened(Ok(info())));
    assert!(m.state.app.tutorial_offered);
    let offers = |m: &Model| {
        m.state
            .toasts
            .iter()
            .filter(|t| t.action == ToastAction::StartTutorial)
            .count()
    };
    assert_eq!(offers(&m), 1);
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::SaveAppSettings(a) if a.tutorial_offered)),
        "remembered across restarts"
    );
    apply(&mut m, Msg::Modal(ModalMsg::ToastDismissed(0)));
    let fx = apply(&mut m, Msg::VaultOpened(Ok(info())));
    assert_eq!(offers(&m), 0, "skipped stays skipped");
    assert!(
        fx.iter()
            .all(|e| !matches!(e, Effect::SaveAppSettings(a) if !a.tutorial_offered))
    );
}

#[test]
fn each_step_opens_its_screen_and_is_done_when_the_user_did_it() {
    let mut m = Model::default();
    apply(&mut m, Msg::VaultOpened(Ok(info())));
    apply(&mut m, Msg::Modal(ModalMsg::ToastClicked(0)));
    let t = m.state.help.tutorial.clone().unwrap();
    assert_eq!((t.step, t.done), (0, false));
    assert_eq!(m.state.screen, FIRST_STEPS[0].screen);

    let mut more = info();
    more.overview.stacks.push(stack("Latin", "latin", 0, 0));
    apply(&mut m, Msg::VaultPolled(Ok(Some(more.clone()))));
    assert!(
        m.state.help.tutorial.as_ref().unwrap().done,
        "a stack was made"
    );

    tutorial(&mut m, TutorialMsg::Next);
    let t = m.state.help.tutorial.clone().unwrap();
    assert_eq!((t.step, t.done), (1, false));
    more.overview.stacks[2].cards = 1;
    apply(&mut m, Msg::VaultPolled(Ok(Some(more))));
    assert!(
        m.state.help.tutorial.as_ref().unwrap().done,
        "a card was made"
    );

    tutorial(&mut m, TutorialMsg::Next);
    assert_eq!(m.state.screen, Screen::Rehearse);
    apply(&mut m, Msg::Navigate(Screen::Home));
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
