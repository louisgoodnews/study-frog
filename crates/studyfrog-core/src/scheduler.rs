//! Which scheduling algorithm an item uses.

/// The spaced-repetition algorithm of a stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Scheduler {
    /// FSRS-6 (default).
    #[default]
    Fsrs,
    /// SM-2, the simple mode.
    Sm2,
}

/// Name used in front-matter and the database.
pub fn scheduler_name(scheduler: Scheduler) -> &'static str {
    match scheduler {
        Scheduler::Fsrs => "fsrs",
        Scheduler::Sm2 => "sm2",
    }
}

/// Inverse of [`scheduler_name`].
pub fn scheduler_from_name(name: &str) -> Option<Scheduler> {
    [Scheduler::Fsrs, Scheduler::Sm2]
        .into_iter()
        .find(|s| scheduler_name(*s) == name)
}
