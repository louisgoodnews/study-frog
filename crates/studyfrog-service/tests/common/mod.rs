#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! A small vault, a database and a driver for the engine.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use rusqlite::Connection;
use studyfrog_core::event::RunEvent;
use studyfrog_core::plan::RunPlan;
use studyfrog_core::review::Rating;
use studyfrog_core::run::{Run, RunMode};
use studyfrog_core::settings::{VaultSettings, vault_settings_default};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::input::{Action, Input};
use studyfrog_rehearsal::queue::candidate::budget_unlimited;
use studyfrog_rehearsal::session::{Session, session_new};
use studyfrog_rehearsal::step::step;
use studyfrog_service::day::day_from;
use studyfrog_service::queue::{NewRun, new_run};
use studyfrog_service::rehearse::{begin_run, record_step, step_context};
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::reindex::reindex;
use studyfrog_store::schema::MIGRATIONS;

pub const DAY: i64 = 86_400_000;
/// Midnight UTC of some day, so `NOW` lies inside day 0.
pub const DAY0: i64 = 1_800_000_000_000 - 1_800_000_000_000 % DAY;
pub const NOW: UnixMs = UnixMs(DAY0 + 10 * 3_600_000);
static COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct Env {
    pub dir: PathBuf,
    pub conn: Connection,
    pub vault: VaultSettings,
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn card_text(id: u128) -> String {
    format!(
        "---\nid: {}\nkind: card\ntitle: Card {id}\n---\nQuestion {id}?\n???\nAnswer {id}\n",
        format_ulid(Uid(id))
    )
}

/// A vault with one stack (settings from `stack_toml`, if given) and cards 11, 12, 13.
pub fn env_with(stack_toml: Option<&str>) -> Env {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("studyfrog-service-{}-{n}", std::process::id()));
    fs::create_dir_all(dir.join("deck")).unwrap();
    let stack = format!(
        "---\nid: {}\nkind: stack\ntitle: Deck\n---\n",
        format_ulid(Uid(1))
    );
    fs::write(dir.join("deck/_stack.md"), stack).unwrap();
    if let Some(toml) = stack_toml {
        fs::write(dir.join("deck/_stack.toml"), toml).unwrap();
    }
    for id in [11, 12, 13] {
        fs::write(dir.join(format!("deck/c{id}.md")), card_text(id)).unwrap();
    }
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    let mut env = Env {
        dir,
        conn,
        vault: vault_settings_default(),
    };
    reload(&mut env);
    env
}

pub fn env() -> Env {
    env_with(None)
}

/// Re-read the vault after files were edited.
pub fn reload(env: &mut Env) {
    reindex(&mut env.conn, &env.dir).unwrap();
}

pub fn build(env: &Env, plan: RunPlan, run: u128, now: UnixMs) -> Run {
    let day = day_from(UnixMs(now.0 - now.0.rem_euclid(DAY)));
    let req = NewRun {
        plan,
        mode: RunMode::Flip,
        uid: Uid(run),
        now,
        day,
        global_budget: budget_unlimited(),
    };
    new_run(&env.conn, req, &env.vault).unwrap()
}

/// Drive a session: every action is stepped and recorded; returns all events in order.
pub fn drive(
    env: &mut Env,
    session: &mut Session,
    wall: UnixMs,
    actions: Vec<(u32, Action)>,
) -> Vec<RunEvent> {
    let mut log = Vec::new();
    for (t_ms, action) in actions {
        let out = step(session, Input { t_ms, action }).unwrap();
        record_step(
            &mut env.conn,
            &step_context(session),
            &out,
            wall,
            &env.vault,
        )
        .unwrap();
        log.extend(out.events);
    }
    log
}

/// Start the run and flip then rate each item with the given ratings, in queue order.
pub fn script(ratings: &[Rating]) -> Vec<(u32, Action)> {
    let mut actions = vec![(0, Action::Start)];
    for (i, r) in ratings.iter().enumerate() {
        let t = 1000 * (i as u32 + 1);
        actions.push((t, Action::Flip));
        actions.push((t + 500, Action::Rate(*r)));
    }
    actions
}

pub fn started(env: &Env, run: Run) -> Session {
    begin_run(&env.conn, &run).unwrap();
    session_new(run)
}
