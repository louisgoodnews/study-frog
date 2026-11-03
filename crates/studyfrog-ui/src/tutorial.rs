//! Guided tutorials as a pure state machine: each step opens the screen it describes and is done when
//! the user has done what it asks, which is read from the state (a count grew since the step began).

use studyfrog_core::event::EventKind;
use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::modal_types::{Toast, ToastAction};
use crate::state::{Screen, State};

/// What a step waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goal {
    /// One more stack.
    Stack,
    /// One more item.
    Item,
    /// A rating in a rehearsal.
    Rating,
    /// One more planned session.
    Session,
}

/// One step of a tutorial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// What to do, short.
    pub title: &'static str,
    /// How to do it.
    pub text: &'static str,
    /// The screen the step opens.
    pub screen: Screen,
    /// What makes it done.
    pub goal: Goal,
}

/// The first-run tutorial.
pub const FIRST_STEPS: [Step; 4] = [
    Step {
        title: "Create a stack",
        text: "A stack holds what you learn for one subject. Press \"+ Create\" in the top bar, choose Stack as the type, give it a name and create it.",
        screen: Screen::Library,
        goal: Goal::Stack,
    },
    Step {
        title: "Create a card",
        text: "Press \"+ Create\" again, keep Card as the type, choose your stack as the parent, and write a question on the front and the answer on the back.",
        screen: Screen::Library,
        goal: Goal::Item,
    },
    Step {
        title: "Rehearse it",
        text: "Pick your stack, start the rehearsal, press Space to see the answer and rate how well you knew it with 1 to 4.",
        screen: Screen::Rehearse,
        goal: Goal::Rating,
    },
    Step {
        title: "Plan a session",
        text: "Choose a day in the calendar and add a study session for your stack, so StudyFrog reminds you.",
        screen: Screen::Calendar,
        goal: Goal::Session,
    },
];

/// The counts the steps compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Facts {
    /// Stacks in the vault.
    pub stacks: usize,
    /// Items in the vault.
    pub items: usize,
    /// The run showing, and the ratings given in it.
    pub ratings: (Option<Uid>, usize),
    /// Planned sessions (`None` until the calendar has been read).
    pub sessions: Option<usize>,
}

/// A tutorial under way.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tutorial {
    /// The step showing (0-based).
    pub step: usize,
    /// The counts when the step began.
    pub start: Facts,
    /// The user has done what the step asks.
    pub done: bool,
}

/// What the user did with the tutorial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialMsg {
    /// Start the first-run tutorial (again).
    Start,
    /// Go to the screen of the step showing.
    Show,
    /// Go on to the next step.
    Next,
    /// End the tutorial.
    Stop,
}

/// The counts in `state` now.
pub fn facts(state: &State) -> Facts {
    let (stacks, items) = state.vault.as_ref().map_or((0, 0), |v| {
        let o = &v.overview;
        let items = o.stacks.iter().map(|s| s.notes + s.cards + s.questions);
        (o.stacks.len(), items.sum::<usize>() + o.inbox_items)
    });
    let ratings = state.rehearse.run.as_ref().map_or((None, 0), |r| {
        let rated = r
            .events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::ItemRated(_)));
        (Some(r.session.run.uid), rated.count())
    });
    let sessions = state.planner.loaded.then_some(state.planner.sessions.len());
    Facts {
        stacks,
        items,
        ratings,
        sessions,
    }
}

/// Whether `goal` is reached between `start` and `now`.
pub fn met(goal: Goal, start: &Facts, now: &Facts) -> bool {
    match goal {
        Goal::Stack => now.stacks > start.stacks,
        Goal::Item => now.items > start.items,
        Goal::Rating => {
            let (run, n) = now.ratings;
            n > 0 && (run != start.ratings.0 || n > start.ratings.1)
        }
        Goal::Session => matches!((start.sessions, now.sessions), (Some(a), Some(b)) if b > a),
    }
}

/// The step showing, if a tutorial is under way.
pub fn current(state: &State) -> Option<(&Tutorial, &'static Step)> {
    let t = state.help.tutorial.as_ref()?;
    FIRST_STEPS.get(t.step).map(|s| (t, s))
}

fn begin(state: State, step: usize) -> (State, Vec<Effect>) {
    let Some(s) = FIRST_STEPS.get(step) else {
        return stop(state);
    };
    let tutorial = Tutorial {
        step,
        start: facts(&state),
        done: false,
    };
    let mut state = state;
    state.help.tutorial = Some(tutorial);
    crate::nav::navigate(state, s.screen)
}

fn stop(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    state.help.tutorial = None;
    (state, Vec::new())
}

/// Handle what the user did with the tutorial.
pub fn update(state: State, msg: TutorialMsg) -> (State, Vec<Effect>) {
    let step = state.help.tutorial.as_ref().map(|t| t.step);
    match (msg, step) {
        (TutorialMsg::Start, _) if state.vault.is_some() => begin(state, 0),
        (TutorialMsg::Show, Some(i)) => {
            let screen = FIRST_STEPS.get(i).map_or(Screen::Home, |s| s.screen);
            crate::nav::navigate(state, screen)
        }
        (TutorialMsg::Next, Some(i)) => begin(state, i + 1),
        (TutorialMsg::Stop, _) => stop(state),
        _ => (state, Vec::new()),
    }
}

/// After every update: mark the step done when its goal is reached, and take the session count as the
/// start once the calendar has been read.
pub fn sync(state: State) -> State {
    let Some((t, step)) = current(&state) else {
        return state;
    };
    let now = facts(&state);
    let mut t = t.clone();
    if t.start.sessions.is_none() {
        t.start.sessions = now.sessions;
    }
    t.done = t.done || met(step.goal, &t.start, &now);
    let mut state = state;
    state.help.tutorial = Some(t);
    state
}

/// Offer the first-run tutorial once, when a vault opens and it was never offered; the caller saves the
/// app settings that remember it.
pub fn offer(state: State) -> State {
    if state.vault.is_none() || state.app.tutorial_offered {
        return state;
    }
    let mut state = state;
    state.app.tutorial_offered = true;
    state.toasts.push(Toast {
        title: "New here? Take the first steps".to_owned(),
        detail: "A short tour: a stack, a card, a rehearsal and the calendar. Dismiss it to skip; it stays in Help.".to_owned(),
        action: ToastAction::StartTutorial,
        expires_ms: None,
    });
    state
}
