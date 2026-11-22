//! The FrogScript reference (STORY-0171): topics in six categories, each with its signature, what it
//! returns, a summary and an example. Data in `wiki_topics`; the search is here. Pure.

use crate::wiki_topics::LANGUAGE;
use crate::wiki_topics_events::EVENTS;
use crate::wiki_topics_library::LIBRARY;

/// The categories the reference is shown in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Verbs, keywords and literals.
    Syntax,
    /// Comparison, arithmetic and logic.
    Operators,
    /// Conditionals, loops and functions.
    ControlFlow,
    /// The functions of the standard library.
    BuiltinFunctions,
    /// The objects a script can reach: vault, calendar, timetable, diary and run.
    VaultObjects,
    /// The events of the instance that start a script listed under `on` in `scripts.toml`.
    LifecycleEvents,
}

/// Every category, in the order the reference lists them.
pub const CATEGORIES: [Category; 6] = [
    Category::Syntax,
    Category::Operators,
    Category::ControlFlow,
    Category::BuiltinFunctions,
    Category::VaultObjects,
    Category::LifecycleEvents,
];

/// The heading the category is listed under.
pub fn label(category: Category) -> &'static str {
    match category {
        Category::Syntax => "Syntax",
        Category::Operators => "Operators",
        Category::ControlFlow => "Control Flow",
        Category::BuiltinFunctions => "Built-in Functions",
        Category::VaultObjects => "Vault Objects",
        Category::LifecycleEvents => "Lifecycle Events",
    }
}

/// One entry of the reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Topic {
    /// The name as written in a script.
    pub name: &'static str,
    /// Where it is listed.
    pub category: Category,
    /// How it is written.
    pub signature: &'static str,
    /// What it gives back, as a type word (`bool`, `Collection`, `Nothing`, ...).
    pub returns: &'static str,
    /// One or two sentences.
    pub summary: &'static str,
    /// A line or two that runs.
    pub example: &'static str,
}

/// Builds a [`Topic`] from its fields in the order they are listed.
macro_rules! topic_entry {
    ($name:expr, $cat:expr, $sig:expr, $ret:expr, $sum:expr, $ex:expr) => {
        Topic {
            name: $name,
            category: $cat,
            signature: $sig,
            returns: $ret,
            summary: $sum,
            example: $ex,
        }
    };
}
pub(crate) use topic_entry;

/// Every topic: the language first, then the library, then the lifecycle events.
pub fn topics() -> Vec<&'static Topic> {
    LANGUAGE
        .iter()
        .chain(LIBRARY.iter())
        .chain(EVENTS.iter())
        .collect()
}

/// The topic with this name, if there is one.
pub fn topic(name: &str) -> Option<&'static Topic> {
    topics().into_iter().find(|t| t.name == name)
}

/// The topics a query finds: the query is matched without case against the name, the signature, the
/// return type, the summary and the category. An empty query finds everything.
pub fn search(query: &str) -> Vec<&'static Topic> {
    let q = query.trim().to_lowercase();
    topics()
        .into_iter()
        .filter(|t| {
            q.is_empty()
                || [t.name, t.signature, t.returns, t.summary, label(t.category)]
                    .iter()
                    .any(|field| field.to_lowercase().contains(&q))
        })
        .collect()
}
