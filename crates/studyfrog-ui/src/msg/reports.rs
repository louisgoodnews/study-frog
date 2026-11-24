//! Report and achievement results.

use studyfrog_service::achievements::AchievementReport;
use studyfrog_service::reports::ReportBundle;

/// Report and achievement results.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgReports {
    /// The reports were computed.
    ReportsBuilt(Result<Box<ReportBundle>, String>),
    /// The achievements were measured.
    AchievementsChecked(Result<AchievementReport, String>),
}
