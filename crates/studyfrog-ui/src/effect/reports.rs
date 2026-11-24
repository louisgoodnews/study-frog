//! Report, explorer and achievement effects.

/// Report, explorer and achievement effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectReports {
    /// Compute the reports from the history.
    BuildReports,
    /// Compute the performance of a scope over a period.
    Explore {
        /// What to look at.
        scope: studyfrog_service::explore::Scope,
        /// How far back.
        period: studyfrog_service::explore::Period,
    },
    /// Measure the achievements and remember new unlocks.
    CheckAchievements,
}
