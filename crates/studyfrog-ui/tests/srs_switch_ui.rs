#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The calibrated-start switch's messages (STORY-0177): asking for it writes it; a failure is said.

use studyfrog_ui::app_settings_ui::{SrsMsg, srs};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::state::{NoticeKind, State};

#[test]
fn asking_for_the_switch_asks_for_the_write() {
    let (_, effects) = srs(State::default(), SrsMsg::CalibratedStart(false));
    assert_eq!(effects, vec![Effect::SetCalibratedStart(false)]);
}

#[test]
fn a_failed_write_is_reported() {
    let (state, effects) = srs(
        State::default(),
        SrsMsg::CalibratedStartApplied(Err("no vault is open".to_owned())),
    );
    assert!(effects.is_empty());
    let notice = state.notice.expect("a notice");
    assert_eq!(notice.kind, NoticeKind::Error);
    assert!(notice.text.contains("no vault is open"));
}
