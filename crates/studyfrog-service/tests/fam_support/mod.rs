#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Familiarity in scheduling: a vault with one card, ratings at each due date, and what they left behind.

use std::fs;
use std::path::PathBuf;

use rusqlite::Connection;
use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_from_days};
use studyfrog_core::event::{EventKind as RunEventKind, RunEvent};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunState;
use studyfrog_core::settings::{VaultSettings, vault_settings_default};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_core::views::{ViewEvent, ViewKind};
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_service::rehearse::{RatingTiming, StepContext, record_step};
use studyfrog_service::schedule::apply_rating;
use studyfrog_service::session::open_vault;
use studyfrog_srs::params::params_default;
use studyfrog_store::schedule::{load_mem_state, load_sm2_state};
use studyfrog_store::sessions::save_session;
use studyfrog_store::views::insert_view;

pub const DAY: i64 = 86_400_000;
pub const START: i64 = 1_700_000_000_000;
pub const ITEM: Uid = Uid(0xA);

pub struct Dir(pub PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn vault(label: &str, stack_toml: Option<&str>) -> Dir {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-famsched-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("s")).unwrap();
    let w = |p: &str, t: &str| fs::write(dir.join(p), t).unwrap();
    w(
        "s/_stack.md",
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    );
    w(
        "s/a.md",
        "---\nid: 0000000000000000000000000A\nkind: card\ntitle: A\n---\nq\n???\na\n",
    );
    if let Some(t) = stack_toml {
        w("s/_stack.toml", t);
    }
    Dir(dir)
}

pub fn rate(
    conn: &mut Connection,
    (seq, rating, timing): (u32, Rating, Option<(u32, u32)>),
    now: i64,
    settings: &VaultSettings,
) {
    let run = Uid(9);
    let event = RunEvent {
        run,
        seq,
        t_ms: 0,
        item: Some(ITEM),
        kind: RunEventKind::ItemRated(rating),
    };
    let out = StepOutput {
        events: vec![event],
        latency: None,
    };
    let ctx = StepContext {
        run,
        effect: ScheduleEffect::UpdateSchedule,
        state: RunState::Running,
        timing: timing.map(|(answer_ms, expected_ms)| RatingTiming {
            answer_ms,
            expected_ms,
        }),
    };
    record_step(conn, &ctx, &out, UnixMs(now), settings).unwrap();
}

/// What a chain of ratings at each due date left behind.
pub struct Chain {
    pub intervals: Vec<i64>,
    pub stored: Vec<f32>,
    pub starts: Vec<i64>,
}

/// How a chain is run.
#[derive(Default)]
pub struct Opts {
    pub sm2: bool,
    pub exam_at: Option<i64>,
    /// Sixty hourly views of the item page between the second and third review.
    pub views: bool,
    /// How long each answer was on screen and how long it should take to read, for every rating.
    pub timing: Option<(u32, u32)>,
}

pub fn chain(dir: &Dir, ratings: &[Rating], settings: &VaultSettings, o: &Opts) -> Chain {
    let (mut session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    if let Some(at) = o.exam_at {
        let entry = Session {
            id: Uid(0xE),
            title: "Exam".into(),
            stack: Some(Uid(1)),
            start: LocalTime {
                date: date_from_days(at.div_euclid(DAY)),
                minute: 600,
            },
            duration_min: 60,
            repeat: Repeat::Never,
            ends: Ends::Forever,
            cancelled: Vec::new(),
            notes: String::new(),
            kind: EventKind::Exam,
            reminders: Vec::new(),
        };
        save_session(&mut session.conn, &entry).unwrap();
    }
    if o.views {
        for h in 0..60 {
            let at = UnixMs(START + 3 * DAY + h * 3_600_000);
            let e = ViewEvent {
                item: ITEM,
                kind: ViewKind::View,
                at,
                active_ms: 20_000,
                expected_ms: None,
            };
            insert_view(&session.conn, &e).unwrap();
        }
    }
    let mut out = Chain {
        intervals: Vec::new(),
        stored: Vec::new(),
        starts: Vec::new(),
    };
    let mut now = START;
    for (i, r) in ratings.iter().enumerate() {
        let before = load_mem_state(&session.conn, ITEM).unwrap();
        rate(&mut session.conn, (i as u32, *r, o.timing), now, settings);
        let (due, stored) = if o.sm2 {
            let s = load_sm2_state(&session.conn, ITEM).unwrap().unwrap();
            (s.due.0, s.state.interval_days as f32)
        } else {
            let m = load_mem_state(&session.conn, ITEM).unwrap().unwrap();
            let plain = apply_rating(before.as_ref(), ITEM, *r, UnixMs(now), &params_default()).1;
            assert_eq!(
                m.state,
                plain.state,
                "familiarity never changes the stored memory state (review {})",
                i + 1
            );
            (m.due.0, m.state.stability)
        };
        out.starts.push(now);
        out.intervals.push((due - now) / DAY);
        out.stored.push(stored);
        now = due;
    }
    out
}

pub fn off() -> VaultSettings {
    VaultSettings {
        familiarity_strength: 0.0,
        ..vault_settings_default()
    }
}

pub fn on() -> VaultSettings {
    vault_settings_default()
}
