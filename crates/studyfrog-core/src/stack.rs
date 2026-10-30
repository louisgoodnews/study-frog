//! Stack: a container of items (the `_stack.md` definition plus its `_stack.toml` settings).

use crate::meta::Meta;
use crate::settings::StackSettings;

/// A stack.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    /// Common fields.
    pub meta: Meta,
    /// The `description` key; empty when absent.
    pub description: String,
    /// Settings from `_stack.toml` (all `None` when the file is absent).
    pub settings: StackSettings,
}
