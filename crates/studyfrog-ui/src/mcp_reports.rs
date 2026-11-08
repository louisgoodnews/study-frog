//! The reports of the MCP server (STORY-0118): `studyfrog://reports/{kind}` and the tools
//! `analytics_get_report` and `forecast_simulate_workload`, all computed by the same service as the
//! Reports screen. Read-only.

use serde_json::{Value, json};
use studyfrog_agent::mcp_route::RESOURCE_NOT_FOUND;
use studyfrog_agent::mcp_types::{
    McpContent, McpError, McpRegistry, McpResourceContent, McpResourceTemplate, McpTool, mcp_error,
    register_resource, register_tool,
};
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::reports::{ReportBundle, build_reports_local};

use crate::mcp_vault::{
    VaultCtx, arg_count, arg_str, internal, json_content, json_result, now_of, read_only,
    tool_error,
};

/// The reports a client can ask for.
pub const REPORT_KINDS: [&str; 6] = [
    "retention",
    "forecast",
    "familiarity",
    "calibration",
    "runs",
    "latency",
];

fn bundle(ctx: &VaultCtx) -> Result<ReportBundle, McpError> {
    let now = now_of(ctx);
    build_reports_local(&ctx.session, now).map_err(internal)
}

fn retention(b: &ReportBundle) -> Value {
    let buckets: Vec<Value> = b.retention.iter().map(|r| json!({
        "from_days": r.from_days, "to_days": r.to_days, "reviews": r.reviews, "recalled": r.recalled,
        "retention": (r.reviews > 0).then(|| f64::from(r.recalled) / f64::from(r.reviews)),
    })).collect();
    json!({"reviews": b.reviews, "buckets": buckets,
        "note": "Recall by the gap since the previous review, measured on the review history."})
}

fn forecast(b: &ReportBundle, days: usize) -> Value {
    let e = b.backtest.error;
    json!({
        "days": days, "reviews_per_day": b.forecast.iter().take(days).collect::<Vec<_>>(),
        "total_reviews": b.forecast.iter().take(days).sum::<f32>(),
        "study_minutes": Value::Null,
        "backtest": {
            "simulated_history": b.backtest.simulated, "days": e.map(|e| e.days),
            "mae_per_day": e.map(|e| e.mae_per_day), "weekly_error": e.map(|e| e.weekly_error),
            "bias": e.map(|e| e.bias),
        },
        "note": "Expected reviews from today on; a study-minute forecast is not built yet.",
    })
}

fn familiarity(b: &ReportBundle) -> Value {
    let f = &b.familiarity;
    let group = |g: &studyfrog_service::fam_report::FamGroup| {
        json!({
            "name": g.name, "folder": g.folder, "percent": g.percent, "items": g.items,
            "least_familiar": g.least.iter().map(|r| json!({"id": format_ulid(r.uid), "title": r.title, "percent": r.percent, "drivers": r.drivers})).collect::<Vec<_>>(),
        })
    };
    json!({"spread_by_tenth": f.spread, "unscored": f.unscored,
        "stacks": f.stacks.iter().map(group).collect::<Vec<_>>(),
        "subjects": f.subjects.iter().map(group).collect::<Vec<_>>()})
}

fn calibration(b: &ReportBundle) -> Value {
    let c = &b.calibration;
    json!({
        "entries_used": c.entries_used, "reviews_evaluated": c.reviews_evaluated,
        "mean_self_score": c.mean_self_score, "mean_accuracy": c.mean_accuracy,
        "mean_familiarity": c.mean_familiarity, "delta": c.delta, "score": c.score,
        "bias": format!("{:?}", c.bias),
        "days": c.buckets.iter().map(|d| json!({"date": studyfrog_core::civil::format_date(d.date), "entries": d.entries, "reviews": d.reviews, "self_score": d.self_score, "accuracy": d.accuracy, "gap": d.gap})).collect::<Vec<_>>(),
    })
}

fn runs(b: &ReportBundle) -> Value {
    let list: Vec<Value> = b
        .runs
        .iter()
        .map(|r| {
            json!({
                "id": format_ulid(r.uid), "mode": format!("{:?}", r.mode).to_lowercase(),
                "pacing": format!("{:?}", r.pacing), "state": r.state, "started_ms": r.started_ms,
                "rated": r.rated, "skipped": r.skipped, "active_ms": r.used_ms,
            })
        })
        .collect();
    json!({"runs": list})
}

fn latency(b: &ReportBundle) -> Value {
    let stats = |s: &studyfrog_report::latency::LatencyStats| json!({"n": s.n, "p50_ms": s.p50, "p90_ms": s.p90, "p95_ms": s.p95, "max_ms": s.max});
    let by_mark: Vec<Value> = b
        .by_mark
        .iter()
        .map(|m| {
            json!({
                "mark": m.mark.map(|d| format!("{d:?}").to_lowercase()), "stats": stats(&m.stats),
            })
        })
        .collect();
    json!({"all": b.latency.as_ref().map(stats), "histogram": b.histogram, "by_difficulty_mark": by_mark})
}

/// The report `kind` as JSON, or `None` for a kind there is none of.
fn report_json(b: &ReportBundle, kind: &str) -> Option<Value> {
    Some(match kind {
        "retention" => retention(b),
        "forecast" => forecast(b, b.forecast.len()),
        "familiarity" => familiarity(b),
        "calibration" => calibration(b),
        "runs" => runs(b),
        "latency" => latency(b),
        _ => return None,
    })
}

fn read_report(
    ctx: &mut VaultCtx,
    uri: &str,
    parts: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let kind = crate::mcp_vault::part(parts, "kind");
    if !REPORT_KINDS.contains(&kind) {
        let known = REPORT_KINDS.join(", ");
        return Err(mcp_error(
            RESOURCE_NOT_FOUND,
            format!("no report {kind}; there are {known}"),
        ));
    }
    let b = bundle(ctx)?;
    Ok(json_content(
        uri,
        &report_json(&b, kind).unwrap_or_default(),
    ))
}

fn analytics(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let kind = arg_str(args, "kind").unwrap_or("retention");
    if kind == "leeches" {
        return Ok(tool_error(
            "Leech diagnostics are not built yet; ask for retention, latency, calibration, familiarity, runs or forecast.",
        ));
    }
    let b = bundle(ctx)?;
    match report_json(&b, kind) {
        Some(v) => Ok(json_result(&json!({"kind": kind, "report": v}))),
        None => Ok(tool_error(format!(
            "no report {kind}; there are {}",
            REPORT_KINDS.join(", ")
        ))),
    }
}

fn workload(ctx: &mut VaultCtx, args: &Value) -> Result<McpContent, McpError> {
    let days = arg_count(args, "days", 30, (1, 30));
    let b = bundle(ctx)?;
    Ok(json_result(&forecast(&b, days)))
}

/// Register the reports resource and tools.
pub fn register(reg: McpRegistry<VaultCtx>) -> McpRegistry<VaultCtx> {
    let reg = register_resource(
        reg,
        McpResourceTemplate {
            uri_template: "studyfrog://reports/{kind}",
            name: "Report",
            description: "A report as JSON: retention, forecast, familiarity, calibration, runs or latency.",
            mime_type: "application/json",
            read: read_report,
        },
    );
    let reg = register_tool(
        reg,
        McpTool {
            name: "analytics_get_report",
            description: "Learning telemetry as JSON: retention by gap, answer latency, self-assessment calibration, familiarity, recent runs or the forecast.",
            input_schema: r#"{"type":"object","properties":{"kind":{"type":"string","enum":["retention","latency","calibration","familiarity","runs","forecast","leeches"],"default":"retention"}}}"#,
            annotations: read_only("Learning report"),
            staged: false,
            call: analytics,
        },
    );
    register_tool(
        reg,
        McpTool {
            name: "forecast_simulate_workload",
            description: "Expected reviews per day for the next days (at most 30) and how far such a forecast was off in a backtest.",
            input_schema: r#"{"type":"object","properties":{"days":{"type":"integer","minimum":1,"maximum":30,"default":30}}}"#,
            annotations: read_only("Workload forecast"),
            staged: false,
            call: workload,
        },
    )
}
