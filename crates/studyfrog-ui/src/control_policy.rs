//! What the control protocol lets through: each command has a class, and the policy says which
//! classes may run. Like `McpPolicy`, destructive commands are refused unless switched on.

use crate::control_cmd::{Command, ItemCmd, RehearseCmd, StackCmd};

/// How much a command may change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// Reads, navigates or changes only what is on screen.
    Read,
    /// Writes to the vault or the settings in a way that can be undone or redone.
    Write,
    /// Deletes or overwrites.
    Destructive,
}

/// What the control protocol lets through. The default lets writes through and refuses destructive
/// commands, like `McpPolicy` with writing on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlPolicy {
    /// Commands of class `Write`.
    pub allow_write: bool,
    /// Commands of class `Destructive`.
    pub allow_destructive: bool,
}

impl Default for ControlPolicy {
    fn default() -> Self {
        policy_default()
    }
}

/// Writes allowed, destructive commands refused.
pub fn policy_default() -> ControlPolicy {
    ControlPolicy {
        allow_write: true,
        allow_destructive: false,
    }
}

/// The class of a command.
pub fn class_of(cmd: &Command) -> Class {
    use Command as C;
    match cmd {
        C::Item(ItemCmd::Delete(_)) => Class::Destructive,
        C::Item(ItemCmd::Open(_)) | C::Stack(StackCmd::Open(_) | StackCmd::Settings(_)) => {
            Class::Read
        }
        C::Item(_) | C::Stack(_) | C::Import { .. } | C::Export(_) => Class::Write,
        C::VaultOpen(_) | C::VaultClose | C::Screenshot(_) => Class::Write,
        C::Rehearse(RehearseCmd::Setup(_) | RehearseCmd::Leave) => Class::Read,
        C::Rehearse(_) => Class::Write,
        _ => Class::Read,
    }
}

/// `Ok` when the policy lets the command through, else the reason.
pub fn allowed(policy: &ControlPolicy, cmd: &Command) -> Result<(), String> {
    match class_of(cmd) {
        Class::Read => Ok(()),
        Class::Write if policy.allow_write => Ok(()),
        Class::Destructive if policy.allow_destructive => Ok(()),
        Class::Write => Err("writing is switched off (start without --control-read-only)".into()),
        Class::Destructive => Err(
            "destructive commands are switched off (start with --control-allow-destructive)".into(),
        ),
    }
}
