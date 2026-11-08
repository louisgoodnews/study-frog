//! Workload against desired retention for the vault and for each stack's own cards, and applying the
//! recommended retention to `studyfrog.toml` or a stack's `_stack.toml` (STORY-0106).

use studyfrog_config::retention_edit::with_request_retention;
use studyfrog_core::uid::Uid;
use studyfrog_report::cmrr::{CmrrReport, cmrr};
use studyfrog_report::sim_engine::SimSetup;
use studyfrog_srs::params::Params;
use studyfrog_store::analytics_rows::StateRow;
use studyfrog_store::error::StoreError;

use crate::analytics_plan::{DeckInput, deck};
use crate::session::{ServiceError, VaultInfo, VaultSession, vault_info};

/// Days each target is simulated.
pub const CMRR_DAYS: usize = 180;
/// Futures per target for the vault (more give a smoother curve).
pub const CMRR_TRAJECTORIES: u32 = 4;
/// Futures per target for each stack (its cards are fewer; the stacks together cost about half the vault's curve).
pub const STACK_TRAJECTORIES: u32 = 2;

/// The curve of one stack's own cards.
#[derive(Debug, Clone, PartialEq)]
pub struct StackCmrr {
    /// The stack.
    pub uid: Uid,
    /// Its folder.
    pub folder: String,
    /// Its title.
    pub title: String,
    /// Its desired retention now.
    pub current: f32,
    /// The curve and the recommendation.
    pub report: CmrrReport,
}

/// The vault's curve.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cmrr {
    /// The vault's desired retention now.
    pub current: f32,
    /// Over every card (its primary stack's state).
    pub vault: CmrrReport,
    /// Over each stack's own cards (stacks without cards are left out).
    pub stacks: Vec<StackCmrr>,
}

fn setup(base: &SimSetup, trajectories: u32) -> SimSetup {
    SimSetup {
        days: CMRR_DAYS,
        trajectories,
        new_days: CMRR_DAYS,
        ..base.clone()
    }
}

/// The curves of the vault (over `primary` rows) and of every stack (over its own `all` rows).
pub fn cmrr_of(
    session: &VaultSession,
    d: &DeckInput,
    (primary, all): (&[StateRow], &[StateRow]),
    base: &SimSetup,
    p: &Params,
) -> Result<Cmrr, StoreError> {
    let (s, per_stack) = (
        setup(base, CMRR_TRAJECTORIES),
        setup(base, STACK_TRAJECTORIES),
    );
    let homes = studyfrog_store::stack_scope::stack_homes(&session.conn)?;
    let stacks = d
        .stacks
        .iter()
        .filter_map(|st| {
            let own: Vec<StateRow> = all.iter().filter(|r| r.stack == st.uid).copied().collect();
            let cards = deck(d, &own, |f| {
                f.stack_path.as_deref() == Some(st.folder.as_str())
            });
            (!cards.is_empty()).then(|| StackCmrr {
                uid: st.uid,
                folder: st.folder.clone(),
                title: st.title.clone(),
                current: homes
                    .get(&st.uid)
                    .and_then(|h| h.request_retention)
                    .unwrap_or(p.request_retention),
                report: cmrr(&cards, &per_stack, p),
            })
        })
        .collect();
    Ok(Cmrr {
        current: p.request_retention,
        vault: cmrr(&deck(d, primary, |_| true), &s, p),
        stacks,
    })
}

/// Write `request_retention = r` into the stack's `_stack.toml` (`folder`) or the vault's `studyfrog.toml` (`None`),
/// keep everything else in the file, read the settings again and return the refreshed vault.
pub fn apply_retention(
    session: &mut VaultSession,
    folder: Option<&str>,
    r: f32,
) -> Result<VaultInfo, ServiceError> {
    if !(0.7..=0.99).contains(&r) {
        return Err(ServiceError::Invalid(format!(
            "{r} is not a retention between 0.7 and 0.99"
        )));
    }
    let (path, table) = match folder {
        Some(f) => (session.root.join(f).join("_stack.toml"), None),
        None => (session.root.join("studyfrog.toml"), Some("srs")),
    };
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    std::fs::write(&path, with_request_retention(&text, r, table))
        .map_err(|e| ServiceError::Io(format!("{}: {e}", path.display())))?;
    studyfrog_store::reindex::reindex(&mut session.conn, &session.root)
        .map_err(ServiceError::Store)?;
    vault_info(&session.root, &session.conn)
}
