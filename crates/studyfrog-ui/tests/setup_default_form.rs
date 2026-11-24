#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A new rehearse setup in the app starts from Spaced unless a practice preset is chosen in it (BUG-0054).
use studyfrog_ui::msg::MsgRehearsal;
use studyfrog_ui::msg::MsgVault;

mod common;

use common::views::open_model;
use studyfrog_core::plan::{Order, ScheduleEffect};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::{Preset, SetupEdit, plan_from_form};
use studyfrog_ui::state::Screen;

fn order_and_effect(m: &studyfrog_ui::model::Model) -> (Order, ScheduleEffect) {
    let plan = plan_from_form(&m.state.rehearse.form, 0).unwrap();
    (plan.order, plan.effect)
}

#[test]
fn a_new_rehearse_setup_starts_spaced_after_a_practice_preset() {
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Rehearse)));
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::Setup(SetupEdit::Preset(
            Preset::EverythingInOrder,
        ))),
    );
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Home)));
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Rehearse)));
    assert_eq!(
        order_and_effect(&m),
        (Order::Due, ScheduleEffect::UpdateSchedule)
    );
}

#[test]
fn a_practice_preset_chosen_in_the_same_setup_stays_practice_only() {
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Home)));
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Rehearse)));
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::Setup(SetupEdit::Preset(
            Preset::EverythingInOrder,
        ))),
    );
    assert_eq!(order_and_effect(&m).1, ScheduleEffect::PracticeOnly);
}
