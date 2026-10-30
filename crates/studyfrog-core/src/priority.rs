//! How much an item matters to its owner. Stored as a word in the item's front matter; an item that
//! sets none takes its stack's (see the rehearsal spec).

/// From least to most important; the default is [`Priority::Medium`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
    /// Least important.
    Lowest,
    /// Less important than usual.
    Low,
    /// The default.
    #[default]
    Medium,
    /// More important than usual.
    High,
    /// Most important.
    Highest,
}

/// Every priority, lowest first.
pub const PRIORITIES: [Priority; 5] = [
    Priority::Lowest,
    Priority::Low,
    Priority::Medium,
    Priority::High,
    Priority::Highest,
];

/// The word stored and shown for a priority.
pub fn priority_name(p: Priority) -> &'static str {
    match p {
        Priority::Lowest => "lowest",
        Priority::Low => "low",
        Priority::Medium => "medium",
        Priority::High => "high",
        Priority::Highest => "highest",
    }
}

/// The priority with this stored word.
pub fn priority_from_name(name: &str) -> Option<Priority> {
    PRIORITIES
        .into_iter()
        .find(|p| priority_name(*p) == name.trim())
}
