//! Whether a run is over and which control of the current item is on screen.
use super::*;

/// True once the run has finished or been aborted.
pub fn is_over(run: &RunView) -> bool {
    matches!(
        run.session.run.state,
        RunState::Finished | RunState::Aborted
    )
}

/// True while the current item is on screen and can be answered or skipped.
pub fn awaiting_reveal(run: &RunView) -> bool {
    run.session.run.state == RunState::Running && run.session.phase == ItemPhase::Shown
}

/// True while the answer is revealed and a rating is expected.
pub fn awaiting_rating(run: &RunView) -> bool {
    run.session.run.state == RunState::Running && run.session.phase == ItemPhase::Revealed
}
