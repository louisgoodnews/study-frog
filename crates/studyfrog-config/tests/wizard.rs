//! The first-run wizard's step model (STORY-0212, TASK-0297).

use studyfrog_config::wizard::{
    Installed, Status, Step, current, goto, is_finished, next, previous, skip, start, status,
};

fn none() -> Installed {
    Installed::default()
}

#[test]
fn the_wizard_starts_at_the_first_optional_step_after_the_vault() {
    assert_eq!(current(&start()), Step::LocalModel);
    assert!(!is_finished(&start()));
}

#[test]
fn next_walks_to_the_summary_and_stops_there() {
    let mut w = start();
    for expected in [
        Step::RemoteProvider,
        Step::Minilm,
        Step::Summary,
        Step::Summary,
    ] {
        w = next(&w);
        assert_eq!(current(&w), expected);
    }
    assert!(is_finished(&w));
}

#[test]
fn previous_stops_at_the_first_optional_step_and_never_returns_to_the_vault() {
    let w = previous(&start());
    assert_eq!(current(&w), Step::LocalModel);
    let w = previous(&next(&next(&start())));
    assert_eq!(current(&w), Step::RemoteProvider);
    let w = previous(&w);
    assert_eq!(current(&w), Step::LocalModel);
}

#[test]
fn skip_records_the_step_and_moves_on() {
    let w = skip(&start());
    assert_eq!(current(&w), Step::RemoteProvider);
    assert_eq!(status(Step::LocalModel, &none(), &w), Status::Skipped);
}

#[test]
fn skipping_the_same_step_twice_records_it_once() {
    let w = previous(&skip(&start()));
    let w = skip(&w);
    assert_eq!(status(Step::LocalModel, &none(), &w), Status::Skipped);
}

#[test]
fn the_vault_and_the_summary_cannot_be_skipped() {
    let w = goto(&start(), Step::Vault);
    assert_eq!(current(&w), Step::LocalModel, "vault is not revisitable");
    let summary = next(&next(&next(&start())));
    assert_eq!(current(&summary), Step::Summary);
    assert_eq!(skip(&summary), summary);
}

#[test]
fn a_satisfied_step_shows_as_done_and_can_still_be_revisited() {
    let installed = Installed {
        local_model: true,
        ..none()
    };
    let w = start();
    assert_eq!(status(Step::LocalModel, &installed, &w), Status::Current);
    let w = next(&w);
    assert_eq!(status(Step::LocalModel, &installed, &w), Status::Done);
    let back = goto(&w, Step::LocalModel);
    assert_eq!(current(&back), Step::LocalModel);
    assert_eq!(status(Step::LocalModel, &installed, &back), Status::Current);
}

#[test]
fn done_wins_over_skipped_when_the_model_is_installed_after_a_skip() {
    let w = skip(&start());
    let installed = Installed {
        local_model: true,
        ..none()
    };
    assert_eq!(status(Step::LocalModel, &installed, &w), Status::Done);
    assert_eq!(status(Step::LocalModel, &none(), &w), Status::Skipped);
}

#[test]
fn the_vault_is_done_when_chosen_and_the_untouched_steps_are_pending() {
    let installed = Installed {
        vault: true,
        ..none()
    };
    let w = start();
    assert_eq!(status(Step::Vault, &installed, &w), Status::Done);
    assert_eq!(status(Step::Minilm, &installed, &w), Status::Pending);
    assert_eq!(status(Step::Summary, &installed, &w), Status::Pending);
}

#[test]
fn goto_to_the_vault_is_ignored() {
    let w = next(&start());
    assert_eq!(goto(&w, Step::Vault), w);
}
