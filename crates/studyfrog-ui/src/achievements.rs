//! The achievements screen's update logic. Pure.

use studyfrog_service::achievements::AchievementReport;

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// A message announcing what just unlocked, if anything.
pub fn unlock_text(report: &AchievementReport) -> Option<String> {
    if report.new.is_empty() {
        return None;
    }
    let names: Vec<String> = report
        .new
        .iter()
        .map(|e| match &e.stack {
            Some(stack) => format!("{} ({stack})", e.title),
            None => e.title.clone(),
        })
        .collect();
    Some(format!("Achievement unlocked: {}", names.join(", ")))
}

/// Keep the report; announce new unlocks. A failed check is reported but never interrupts anything.
pub fn checked(state: State, result: Result<AchievementReport, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(report) => {
            let notice = unlock_text(&report)
                .map(|text| Notice {
                    kind: NoticeKind::Info,
                    text,
                })
                .or(state.notice.clone());
            (
                State {
                    achievements: Some(report),
                    notice,
                    ..state
                },
                Vec::new(),
            )
        }
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not check achievements: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}
