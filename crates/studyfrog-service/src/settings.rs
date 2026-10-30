//! Resolving vault and stack settings into what the scheduling code needs.

use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::settings::VaultSettings;
use studyfrog_srs::params::{DEFAULT_WEIGHTS, Params, params_new};
use studyfrog_store::settings::ItemHome;

/// FSRS parameters for an item: vault weights and interval cap, with the stack's retention override.
pub fn params_for(vault: &VaultSettings, retention_override: Option<f32>) -> Params {
    let weights = vault.weights.unwrap_or(DEFAULT_WEIGHTS);
    params_new(
        weights,
        retention_override.unwrap_or(vault.request_retention),
        vault.maximum_interval_days,
    )
}

/// The scheduler an item uses: its stack's choice, else the vault default.
pub fn scheduler_for(vault: &VaultSettings, home: Option<&ItemHome>) -> Scheduler {
    home.and_then(|h| h.scheduler).unwrap_or(vault.scheduler)
}
