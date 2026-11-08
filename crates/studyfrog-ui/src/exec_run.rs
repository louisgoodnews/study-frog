//! Effects of a rehearsal run: create it, load item content, save steps.

use studyfrog_core::plan::RunPlan;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_service::achievements::check_achievements;
use studyfrog_service::compare::{record_answer_score, score_item_answer_with};
use studyfrog_service::content::load_item_content;
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::home::home_summary;
use studyfrog_service::rehearse::{StepContext, record_step, start_run};
use studyfrog_service::reports::build_reports_local;
use studyfrog_service::rerate::rerate_latest;
use studyfrog_service::session::{VaultSession, current_settings};
use studyfrog_service::timezone::{local_day_window, local_offset_minutes};
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::models::{Opened, open_runtime};

use crate::exec::Worker;
use crate::home_ui::HomeMsg;
use crate::msg::Msg;
use crate::run_state::Scored;

fn millis(now_ns: i64) -> i64 {
    now_ns / 1_000_000
}

/// Create and store a run.
pub fn start(w: &Worker, plan: RunPlan, mode: RunMode, now_ns: i64) -> Msg {
    let Some(session) = &w.session else {
        return Msg::RunReady(Err("no vault is open".to_owned()));
    };
    let now_ms = millis(now_ns);
    let result = fresh_id_source(u64::try_from(now_ms).unwrap_or(0)).and_then(|ids| {
        start_run(
            session,
            plan,
            mode,
            UnixMs(now_ms),
            local_day_window(UnixMs(now_ms)),
            ids,
        )
    });
    Msg::RunReady(result.map_err(|e| e.to_string()))
}

/// Start a practice run of exactly `items`.
pub fn drill(
    session: &studyfrog_service::session::VaultSession,
    items: &[Uid],
    now_ns: i64,
) -> Msg {
    let now = UnixMs(millis(now_ns));
    let result = fresh_id_source(u64::try_from(now.0).unwrap_or(0)).and_then(|ids| {
        studyfrog_service::drill::start_drill(
            session,
            items,
            RunMode::Flip,
            (now, local_day_window(now)),
            ids,
        )
    });
    Msg::RunReady(result.map_err(|e| e.to_string()))
}

/// Read an item's content.
pub fn load_item(w: &Worker, uid: Uid) -> Msg {
    match &w.session {
        Some(session) => Msg::ItemLoaded(
            uid,
            load_item_content(session, uid).map_err(|e| e.to_string()),
        ),
        None => Msg::ItemLoaded(uid, Err("no vault is open".to_owned())),
    }
}

/// Save one engine step.
pub fn record(w: &mut Worker, ctx: &StepContext, out: &StepOutput, now_ns: i64) -> Msg {
    let Some(session) = &mut w.session else {
        return Msg::StepRecorded(Err("no vault is open".to_owned()));
    };
    let vault = current_settings(session);
    let result = record_step(&mut session.conn, ctx, out, UnixMs(millis(now_ns)), &vault);
    Msg::StepRecorded(result.map_err(|e| e.to_string()))
}

/// Compare a typed answer with the item's reference (offline, with the built-in lexical runtime).
pub fn score(
    w: &mut Worker,
    (run, uid): (Uid, Uid),
    (answer, reference): &(String, Option<String>),
    words_only: bool,
    now_ns: i64,
) -> Msg {
    let (models, loader) = (w.models_dir.clone(), w.loader);
    let chosen: Opened = (Box::new(Lexical), Some("chosen in the settings".to_owned()));
    let (runtime, why) = if words_only {
        &chosen
    } else {
        w.slm
            .get_or_insert_with(|| open_runtime(models.as_deref(), loader))
    };
    let result = match &w.session {
        Some(session) => {
            score_item_answer_with(session, runtime.as_ref(), uid, answer, reference.as_deref())
                .map(|s| match why {
                    Some(why) => Scored {
                        engine: format!("{} ({why})", s.engine),
                        ..s
                    },
                    None => s,
                })
                .map(|s| remember(session, run, uid, s, now_ns))
                .map_err(|e| e.to_string())
        }
        None => Err("no vault is open".to_owned()),
    };
    Msg::AnswerScored(uid, result)
}

/// Store the comparison with its run; a failure is noted in the engine text, never hidden.
fn remember(session: &VaultSession, run: Uid, item: Uid, s: Scored, now_ns: i64) -> Scored {
    match record_answer_score(session, run, item, &s, UnixMs(millis(now_ns))) {
        Ok(()) => s,
        Err(e) => Scored {
            engine: format!("{} (not saved: {e})", s.engine),
            ..s
        },
    }
}

/// Replace the rating of an item's latest review and rebuild its schedule.
pub fn rerate(w: &mut Worker, item: Uid, expected: Rating, to: Rating) -> Msg {
    let result = match &mut w.session {
        Some(session) => rerate_latest(session, item, expected, to).map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Rerated(item, result)
}

/// Measure the achievements at the current local time.
pub fn achievements(w: &mut Worker, now_ns: i64) -> Msg {
    let now = UnixMs(millis(now_ns));
    let result = match &mut w.session {
        Some(session) => {
            check_achievements(session, now, local_offset_minutes(now)).map_err(|e| e.to_string())
        }
        None => Err("no vault is open".to_owned()),
    };
    Msg::AchievementsChecked(result)
}

/// Compute the reports at the current local time.
pub fn reports(w: &Worker, now_ns: i64) -> Msg {
    let now = UnixMs(millis(now_ns));
    let result = match &w.session {
        Some(session) => build_reports_local(session, now).map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::ReportsBuilt(result)
}

/// Write the reports as a CSV file into the vault's `exports` folder; the path, or why not.
pub fn report_csv_file(w: &Worker, now_ns: i64) -> Result<String, String> {
    use studyfrog_core::civil::{date_from_days, format_date};
    use studyfrog_service::report_csv::{data_of, report_csv, report_csv_name};
    let session = w.session.as_ref().ok_or("no vault is open")?;
    let now = UnixMs(millis(now_ns));
    let bundle = build_reports_local(session, now).map_err(|e| e.to_string())?;
    let day = format_date(date_from_days(now.0.div_euclid(86_400_000)));
    let vault = session.root.file_name().map_or_else(
        || "vault".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let dir = session.root.join("exports");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let target = dir.join(report_csv_name(&vault, &day));
    std::fs::write(&target, report_csv(&data_of(&bundle)))
        .map_err(|e| format!("{}: {e}", target.display()))?;
    Ok(target.display().to_string())
}

/// Read the home screen's numbers at the current local time.
pub fn home(w: &Worker, now_ns: i64) -> Msg {
    let now = UnixMs(millis(now_ns));
    let result = match &w.session {
        Some(session) => home_summary(
            session,
            now,
            local_offset_minutes(now),
            local_day_window(now),
        )
        .map(Box::new)
        .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Home(HomeMsg::Loaded(result))
}

/// Compute the performance explorer for a scope and period at the current local time.
pub fn explorer(
    w: &Worker,
    (scope, period): (
        studyfrog_service::explore::Scope,
        studyfrog_service::explore::Period,
    ),
    now_ns: i64,
) -> Msg {
    let now = UnixMs(now_ns / 1_000_000);
    let result = match &w.session {
        Some(session) => studyfrog_service::explore::explore(
            session,
            (scope, period),
            now,
            local_offset_minutes(now),
        )
        .map(Box::new)
        .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Explorer(crate::explorer_ui::ExplorerMsg::Loaded(result))
}
