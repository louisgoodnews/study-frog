//! The exam pacer of a stack as a card: the daily quota of new cards and reviews to be ready on the exam day
//! (STORY-0105). Shown on the Workload tab of the reports and on the stack's Summary and Reports tabs.

use eframe::egui;
use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_service::analytics_plan::StackPace;

use crate::view::kit;

/// The lines of one pacer, as plain sentences.
pub fn lines(s: &StackPace) -> Vec<String> {
    let p = &s.pace;
    let verdict = if p.on_track {
        format!(
            "On track: {:.0} % of the cards would be at 90 % recall on the day.",
            p.ready_share * 100.0
        )
    } else {
        format!(
            "Behind: only {:.0} % of the cards would be at 90 % recall on the day; study more each day or start earlier.",
            p.ready_share * 100.0
        )
    };
    vec![
        format!(
            "{}: exam on {}, in {} day(s).",
            s.title,
            format_date(date_from_days(s.exam_day)),
            p.days_left
        ),
        format!(
            "Start {} new card(s) a day for {} day(s) ({} never seen), then keep the last {} day(s) for consolidation.",
            p.required_new_per_day,
            p.study_days,
            p.unseen_items,
            p.days_left - p.study_days
        ),
        format!(
            "About {:.0} review(s) and {:.1} minute(s) a day; {} card(s) would fall under 90 % without another review, about {:.0} consolidation review(s) at the end.",
            p.reviews_per_day, p.minutes_per_day, p.volatile_items, p.consolidation_reviews
        ),
        verdict,
    ]
}

/// The card.
pub fn draw(s: &StackPace, ui: &mut egui::Ui) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        for (i, line) in lines(s).iter().enumerate() {
            if i == 0 {
                ui.strong(line);
            } else {
                ui.label(line);
            }
        }
    });
}

/// The pacer of the stack at `folder`, if it has an exam ahead.
pub fn of_stack<'a>(state: &'a crate::state::State, folder: &str) -> Option<&'a StackPace> {
    state
        .reports
        .as_ref()?
        .analytics
        .plan
        .pacers
        .iter()
        .find(|p| p.folder == folder)
}
