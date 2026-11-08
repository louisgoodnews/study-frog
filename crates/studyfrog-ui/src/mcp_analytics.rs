//! JSON of the EPIC-034 analytics for the MCP server (STORY-0118): true retention and the forgetting curves by
//! stability tier, leeches and confusion pairs, and the simulated workload with study minutes. Read-only.

use serde_json::{Value, json};
use studyfrog_md::ulid::format_ulid;
use studyfrog_report::forecast_sim::load_total;
use studyfrog_report::retention_survival::{RateCount, SurvivalReport, predicted_of, rate_of};
use studyfrog_service::analytics::Analytics;

fn count(c: &RateCount) -> Value {
    json!({"reviews": c.reviews, "recalled": c.recalled, "retention": rate_of(c), "predicted": predicted_of(c)})
}

/// True retention, retention by maturity and timing, and the measured and predicted curve of each tier.
pub fn survival(s: &SurvivalReport) -> Value {
    let tiers: Vec<Value> = s
        .tiers
        .iter()
        .map(|t| {
            let bins: Vec<Value> = t
                .bins
                .iter()
                .map(|b| {
                    json!({"from_ratio": b.from_ratio, "to_ratio": b.to_ratio.is_finite().then_some(b.to_ratio), "measured": count(&b.count)})
                })
                .collect();
            json!({
                "from_stability_days": t.from_days, "to_stability_days": t.to_days.is_finite().then_some(t.to_days),
                "all": count(&t.total), "rmse": t.rmse, "by_elapsed_over_stability": bins,
            })
        })
        .collect();
    json!({
        "true_retention": count(&s.true_retention), "young": count(&s.young), "mature": count(&s.mature),
        "early": count(&s.early), "on_time": count(&s.on_time), "overdue": count(&s.overdue),
        "stability_tiers": tiers, "memory_scale": s.memory_scale,
        "half_life_days": s.half_life_days, "model_half_life_days": s.model_half_life_days,
        "note": "True retention: mature items (stability >= 21 days) reviewed within 20 % of their interval. Curves: measured recall against the FSRS prediction R(t, S) by t/S.",
    })
}

/// Leeches with their reasons and the pairs that seem to interfere.
pub fn leeches(a: &Analytics) -> Value {
    let leeches: Vec<Value> = a
        .leeches
        .iter()
        .map(|r| {
            let l = &r.leech;
            json!({
                "id": format_ulid(r.uid), "title": r.title, "stack_folder": r.folder, "lapses": l.lapses,
                "hard_or_again_streak": l.streak, "reviews": l.reviews, "time_burden": l.time_burden,
                "stability_per_review_days": l.efficiency, "score": l.score,
                "reasons": l.reasons.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>(),
            })
        })
        .collect();
    let pairs: Vec<Value> = a
        .pairs
        .iter()
        .map(|p| {
            let x = &p.pair;
            json!({
                "a": {"id": format_ulid(p.a.0), "title": p.a.1, "stack_folder": p.folders.0},
                "b": {"id": format_ulid(p.b.0), "title": p.b.1, "stack_folder": p.folders.1},
                "word_similarity": x.similarity, "close_lapses": x.co_lapses, "shared_tags": x.shared_tags,
                "linked": x.linked, "same_stack": x.same_group, "score": x.score,
            })
        })
        .collect();
    json!({"leeches": leeches, "confusion_pairs": pairs,
        "note": "Leeches: >= 4 lapses, a Hard/Again streak of 3, 3x the stack's median time, or >= 6 reviews under 7 days of stability."})
}

/// The simulated workload of the next `days` days.
pub fn workload(a: &Analytics, days: usize) -> Value {
    let plan = &a.plan;
    let f = &plan.forecast[..days.min(plan.forecast.len())];
    let per_day: Vec<Value> = f
        .iter()
        .map(|d| {
            json!({"reviews": load_total(d), "new": d.new, "learning": d.learning, "young": d.young, "mature": d.mature,
                "p10": d.p10, "p50": d.p50, "p90": d.p90, "minutes": d.minutes, "minutes_p10": d.minutes_p10, "minutes_p90": d.minutes_p90})
        })
        .collect();
    let pacers: Vec<Value> = plan
        .pacers
        .iter()
        .map(|s| {
            let p = &s.pace;
            json!({"stack_folder": s.folder, "title": s.title, "exam_day": studyfrog_core::civil::format_date(studyfrog_core::civil::date_from_days(s.exam_day)),
                "days_left": p.days_left, "required_new_per_day": p.required_new_per_day, "reviews_per_day": p.reviews_per_day,
                "minutes_per_day": p.minutes_per_day, "unseen_items": p.unseen_items, "volatile_items": p.volatile_items,
                "ready_share": p.ready_share, "on_track": p.on_track})
        })
        .collect();
    json!({
        "days": f.len(), "reviews_per_day": f.iter().map(load_total).collect::<Vec<_>>(),
        "study_minutes": f.iter().map(|d| d.minutes).collect::<Vec<_>>(),
        "total_reviews": f.iter().map(load_total).sum::<f32>(), "total_minutes": f.iter().map(|d| d.minutes).sum::<f32>(),
        "per_day": per_day, "miss_probability": plan.miss_probability, "new_per_day": plan.new_per_day,
        "exam_pacers": pacers,
        "recommended_retention": plan.cmrr.vault.recommended, "current_retention": plan.cmrr.current,
    })
}
