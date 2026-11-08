//! The simulated forecast of the vault and the exam pacer of every stack with an exam or due date ahead
//! (STORY-0105), from the stored scheduler states, the review history and the run log.

use std::collections::{HashMap, HashSet};

use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::date_to_days;
use studyfrog_core::review::Rating;
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::uid::Uid;
use studyfrog_report::exam_pacer::{ExamPace, exam_pace, pacer_input};
use studyfrog_report::forecast_sim::{DayLoad, simulate_forecast};
use studyfrog_report::sim_engine::{Pace, RatingMix, SimCard, SimSetup};
use studyfrog_srs::params::Params;
use studyfrog_store::analytics_rows::{ItemFact, StateRow, load_state_rows};
use studyfrog_store::error::StoreError;
use studyfrog_store::overview::{StackRow, load_overview};
use studyfrog_store::sessions::load_sessions;

use crate::analytics_deck::{
    card_of, folders, latest_per_item, miss_probability, new_card, new_per_day, paces, rating_mix,
};
use crate::session::{VaultSession, current_settings};

/// Days the simulated forecast covers (the screen shows 30, 90 or all 365).
pub const SIM_DAYS: usize = 365;
/// Futures simulated for the band.
pub const SIM_TRAJECTORIES: u32 = 16;

/// The exam pacer of one stack.
#[derive(Debug, Clone, PartialEq)]
pub struct StackPace {
    /// The stack.
    pub uid: Uid,
    /// Its folder.
    pub folder: String,
    /// Its title.
    pub title: String,
    /// The exam or due day (days since 1970-01-01).
    pub exam_day: i64,
    /// The plan.
    pub pace: ExamPace,
}

/// The forecast and the pacers.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Plan {
    /// Expected reviews and minutes per day for [`SIM_DAYS`] days from today.
    pub forecast: Vec<DayLoad>,
    /// The chance of a missed day the forecast assumed.
    pub miss_probability: f32,
    /// New cards a day the forecast assumed.
    pub new_per_day: f32,
    /// The rating mix of the learner.
    pub mix: Option<RatingMix>,
    /// One per stack with an exam or due date ahead.
    pub pacers: Vec<StackPace>,
}

/// What the deck is built from.
pub struct DeckInput<'a> {
    /// Cards and questions of the index.
    pub facts: &'a [ItemFact],
    /// Paces and the pace index of each folder.
    pub paces: &'a (Vec<Pace>, HashMap<String, u16>),
    /// Items whose last rating was Again.
    pub relearning: &'a HashSet<Uid>,
    /// Stacks by folder, for their scheduler.
    pub stacks: &'a [StackRow],
    /// The vault's scheduler.
    pub scheduler: Scheduler,
    /// Local UTC offset now.
    pub offset: i32,
}

fn pace_index(d: &DeckInput, folder: &str) -> u16 {
    d.paces.1.get(folder).copied().unwrap_or(0)
}

fn sm2_in(d: &DeckInput, folder: &str) -> bool {
    let own = d
        .stacks
        .iter()
        .find(|s| s.folder == folder)
        .and_then(|s| s.scheduler);
    own.unwrap_or(d.scheduler) == Scheduler::Sm2
}

/// The cards of `facts` that pass `keep`: stored states from `rows`, the rest new.
pub fn deck(d: &DeckInput, rows: &[StateRow], keep: impl Fn(&ItemFact) -> bool) -> Vec<SimCard> {
    let by_item = latest_per_item(rows);
    d.facts
        .iter()
        .filter(|f| keep(f))
        .map(|f| {
            let folder = f.stack_path.clone().unwrap_or_default();
            let pace = pace_index(d, &folder);
            match by_item.get(&f.uid) {
                Some(r) => card_of(r, d.offset, d.relearning.contains(&f.uid), pace),
                None => new_card(sm2_in(d, &folder), pace),
            }
        })
        .collect()
}

fn in_stack(folder: &str, stack: &str) -> bool {
    folder == stack || folder.starts_with(&format!("{stack}/"))
}

fn exam_days(session: &VaultSession, today: i64) -> Result<HashMap<Uid, i64>, StoreError> {
    let mut out: HashMap<Uid, i64> = HashMap::new();
    for s in load_sessions(&session.conn)? {
        let Some(stack) = s
            .stack
            .filter(|_| matches!(s.kind, EventKind::Exam | EventKind::Deadline))
        else {
            continue;
        };
        let day = date_to_days(s.start.date);
        if day > today && out.get(&stack).is_none_or(|d| day < *d) {
            out.insert(stack, day);
        }
    }
    Ok(out)
}

fn pacers(
    session: &VaultSession,
    d: &DeckInput,
    mix: RatingMix,
    today: i64,
    p: &Params,
) -> Result<Vec<StackPace>, StoreError> {
    let all = load_state_rows(&session.conn, false)?;
    let mut out = Vec::new();
    for (stack, exam_day) in exam_days(session, today)? {
        let Some(row) = d.stacks.iter().find(|s| s.uid == stack) else {
            continue;
        };
        let own: Vec<StateRow> = all.iter().filter(|r| r.stack == stack).copied().collect();
        let cards = deck(d, &own, |f| {
            in_stack(f.stack_path.as_deref().unwrap_or(""), &row.folder)
        });
        let input = studyfrog_report::exam_pacer::PacerInput {
            pace: d.paces.0[usize::from(pace_index(d, &row.folder))],
            ..pacer_input(today, exam_day, mix)
        };
        if let Some(pace) = exam_pace(&cards, &input, p) {
            out.push(StackPace {
                uid: stack,
                folder: row.folder.clone(),
                title: row.title.clone(),
                exam_day,
                pace,
            });
        }
    }
    out.sort_by(|a, b| a.exam_day.cmp(&b.exam_day).then(a.folder.cmp(&b.folder)));
    Ok(out)
}

/// The forecast and the pacers at `today`.
pub fn plan_of(
    session: &VaultSession,
    h: &crate::analytics::History,
    samples: &[studyfrog_report::session_log::SessionSample],
    starts: &HashMap<Uid, i64>,
    (today, offset): (i64, i32),
    p: &Params,
) -> Result<Plan, StoreError> {
    let facts = studyfrog_store::analytics_rows::load_item_facts(&session.conn)?;
    let pace_list = paces(h.rows, samples, starts, &folders(&facts));
    let mut last: HashMap<Uid, Rating> = HashMap::new();
    h.rows
        .iter()
        .for_each(|r| _ = last.insert(r.item, r.rating));
    let relearning: HashSet<Uid> = last
        .into_iter()
        .filter(|x| x.1 == Rating::Again)
        .map(|x| x.0)
        .collect();
    let stacks = load_overview(&session.conn)?.stacks;
    let d = DeckInput {
        facts: &facts,
        paces: &pace_list,
        relearning: &relearning,
        stacks: &stacks,
        scheduler: current_settings(session).scheduler,
        offset,
    };
    let cards = deck(&d, &load_state_rows(&session.conn, true)?, |_| true);
    let mix = rating_mix(h.rows);
    let setup = SimSetup {
        today,
        days: SIM_DAYS,
        trajectories: SIM_TRAJECTORIES,
        seed: 1,
        miss_probability: miss_probability(h.rows, today, offset),
        new_per_day: new_per_day(h.rows, today, offset),
        new_days: SIM_DAYS,
        mix,
        paces: pace_list.0.clone(),
    };
    Ok(Plan {
        forecast: simulate_forecast(&cards, &setup, p),
        miss_probability: setup.miss_probability,
        new_per_day: setup.new_per_day,
        mix: Some(mix),
        pacers: pacers(session, &d, mix, today, p)?,
    })
}
