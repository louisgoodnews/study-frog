//! The figures each reports tab shows, as JSON. Pure: one function per tab, each reading only what
//! the tab draws.

use serde_json::{Value, json};
use studyfrog_report::chronotype::ClockWindow;
use studyfrog_service::reports::ReportBundle;

use crate::explorer_ui::ReportTab;
use crate::state::State;

/// Rows of a list (leeches, runs) a tab answers with at most.
const MAX_LIST: usize = 20;
/// Days of forecast a memory answer carries at most.
const FORECAST_DAYS: usize = 30;

/// The figures of one tab.
pub fn figures(state: &State, r: &ReportBundle, tab: ReportTab) -> Value {
    match tab {
        ReportTab::Performance => performance(state),
        ReportTab::Memory => memory(r),
        ReportTab::Familiarity => familiarity(r),
        ReportTab::Calibration => calibration(r),
        ReportTab::Cognitive => cognitive(r),
        ReportTab::Runs => runs(r),
        ReportTab::Habits => habits(r),
        ReportTab::Leeches => leeches(r),
        ReportTab::Workload => workload(state, r),
    }
}

fn performance(state: &State) -> Value {
    let Some(data) = state.explorer.data.as_ref() else {
        return Value::Null;
    };
    let p = &data.perf;
    json!({
        "reviews": p.reviews, "items": p.items,
        "ratings": {"again": p.ratings[0], "hard": p.ratings[1], "good": p.ratings[2], "easy": p.ratings[3]},
        "median_ms": p.median_ms, "total_ms": p.total_ms, "last_ms": p.last_ms,
    })
}

fn memory(r: &ReportBundle) -> Value {
    let retention: Vec<Value> = r
        .retention
        .iter()
        .map(|b| {
            json!({
                "from_days": b.from_days,
                "to_days": (b.to_days != u32::MAX).then_some(b.to_days),
                "reviews": b.reviews, "recalled": b.recalled,
            })
        })
        .collect();
    let latency = r
        .latency
        .as_ref()
        .map(|l| json!({"n": l.n, "p50": l.p50, "p90": l.p90, "p95": l.p95}));
    let forecast: Vec<f32> = r.forecast.iter().take(FORECAST_DAYS).copied().collect();
    json!({"retention": retention, "latency": latency, "forecast": forecast})
}

fn familiarity(r: &ReportBundle) -> Value {
    let f = &r.familiarity;
    json!({"spread": f.spread, "unscored": f.unscored, "stacks": f.stacks.len(), "subjects": f.subjects.len()})
}

fn calibration(r: &ReportBundle) -> Value {
    let c = &r.calibration;
    json!({
        "entries_used": c.entries_used, "reviews_evaluated": c.reviews_evaluated,
        "mean_self_score": c.mean_self_score, "mean_accuracy": c.mean_accuracy,
        "mean_familiarity": c.mean_familiarity, "delta": c.delta, "score": c.score,
        "bias": format!("{:?}", c.bias),
    })
}

fn cognitive(r: &ReportBundle) -> Value {
    let p = &r.cognitive;
    let calibration = p
        .calibration
        .map(|c| json!({"easy_share": c.easy_share, "in_time_share": c.in_time_share}));
    json!({
        "answers": p.answers, "timed": p.timed, "velocity": p.velocity,
        "calibration": calibration, "scored": p.scored,
        "weekly_pass": p.weekly_pass,
    })
}

fn runs(r: &ReportBundle) -> Value {
    let latest: Vec<Value> = r
        .runs
        .iter()
        .take(MAX_LIST)
        .map(|run| {
            json!({
                "uid": crate::control_state::ulid(run.uid), "mode": format!("{:?}", run.mode),
                "pacing": format!("{:?}", run.pacing), "state": run.state,
                "started_ms": run.started_ms, "rated": run.rated,
                "skipped": run.skipped, "used_ms": run.used_ms,
            })
        })
        .collect();
    json!({"count": r.runs.len(), "latest": latest})
}

fn window(w: Option<&ClockWindow>) -> Value {
    w.map(|w| {
        json!({"start_hour": w.start_hour, "hours": w.hours, "retention": w.retention,
               "median_ms": w.median_ms, "reviews": w.reviews})
    })
    .unwrap_or(Value::Null)
}

fn habits(r: &ReportBundle) -> Value {
    let chrono = &r.analytics.chronotype;
    let fatigue = &r.analytics.fatigue;
    json!({
        "peak": window(chrono.peak.as_ref()),
        "trough": window(chrono.trough.as_ref()),
        "best_weekday": chrono.best_weekday,
        "worst_weekday": chrono.worst_weekday,
        "hour_reviews": chrono.hours.iter().map(|h| h.reviews).collect::<Vec<_>>(),
        "fatigue_sessions": fatigue.sessions,
        "fatigue_baseline_reviews": fatigue.baseline.reviews,
    })
}

fn leeches(r: &ReportBundle) -> Value {
    let a = &r.analytics;
    let items: Vec<Value> = a
        .leeches
        .iter()
        .take(MAX_LIST)
        .map(|l| {
            json!({
                "uid": crate::control_state::ulid(l.uid), "title": l.title, "folder": l.folder,
                "lapses": l.leech.lapses, "streak": l.leech.streak,
            })
        })
        .collect();
    json!({"count": a.leeches.len(), "items": items, "pairs": a.pairs.len()})
}

fn workload(state: &State, r: &ReportBundle) -> Value {
    let horizon = match state.explorer.horizon {
        0 => 30,
        h => usize::from(h),
    };
    let days: Vec<Value> = r
        .analytics
        .plan
        .forecast
        .iter()
        .take(horizon)
        .map(
            |d| json!({"new": d.new, "learning": d.learning, "young": d.young, "mature": d.mature}),
        )
        .collect();
    json!({
        "horizon_days": horizon, "days": days,
        "miss_probability": r.analytics.plan.miss_probability,
        "new_per_day": r.analytics.plan.new_per_day,
    })
}
