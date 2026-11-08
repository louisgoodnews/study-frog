#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! A fixture vault (a card, a question with synonyms and a rubric, a note) and an exam played through the engine.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::exam::exam_plan;
use studyfrog_core::plan::{Order, plan_sequential};
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::settings::vault_settings_default;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_rehearsal::input::{Action, Input};
use studyfrog_rehearsal::session::{Session, session_new};
use studyfrog_rehearsal::step::step;
use studyfrog_service::compare::{Scored, record_answer_score, score_item_answer};
use studyfrog_service::create::IdSource;
use studyfrog_service::rehearse::{record_step, start_run, step_context};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_slm::lexical::Lexical;
use studyfrog_store::run_load::load_run;

pub const CARD: Uid = Uid(10);
pub const QUESTION: Uid = Uid(11);
pub const NOTE: Uid = Uid(12);

pub struct Dir(pub PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn fixture_vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-exam-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bio")).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    let files = [
        (
            "bio/_stack.md",
            format!("---\nid: {}\nkind: stack\ntitle: Bio\n---\n", id(1)),
        ),
        (
            "bio/a-card.md",
            format!(
                "---\nid: {}\nkind: card\ntitle: Capital\n---\nCapital of France?\n???\nParis\n",
                id(10)
            ),
        ),
        (
            "bio/b-question.md",
            format!(
                "---\nid: {}\nkind: question\ntitle: Energy\naccepted:\n  - powerhouse of the cell\nrubric:\n  - mitochondria\n  - ATP\n---\nWhat makes energy in a cell?\n???\nThe mitochondria make ATP\n",
                id(11)
            ),
        ),
        (
            "bio/c-note.md",
            format!("---\nid: {}\nkind: note\ntitle: Note\n---\nbody\n", id(12)),
        ),
    ];
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    Dir(dir)
}

pub fn open(dir: &Dir) -> VaultSession {
    open_vault(dir.0.to_str().unwrap()).unwrap().0
}

pub fn start_exam_with(s: &VaultSession, n: u8) -> Run {
    let plan = exam_plan(plan_sequential()).unwrap();
    let now = UnixMs(1_800_000_000_000);
    let day = studyfrog_service::day::day_from(UnixMs(now.0 - now.0.rem_euclid(86_400_000)));
    let ids = IdSource {
        now_ms: 1_800_000_000_000 + u64::from(n),
        random: [n; 10],
    };
    start_run(s, plan, RunMode::Exam, now, day, ids).unwrap()
}

pub fn start_exam(s: &VaultSession) -> Run {
    start_exam_with(s, 1)
}

fn apply(s: &mut VaultSession, session: &mut Session, t: u32, action: Action) {
    let out = step(session, Input { t_ms: t, action }).unwrap();
    let vault = vault_settings_default();
    record_step(
        &mut s.conn,
        &step_context(session),
        &out,
        UnixMs(1_800_000_000_000),
        &vault,
    )
    .unwrap();
}

/// Start the stored run in the engine (its queue as stored) and keep the session for more steps.
pub fn engine(run: &Run) -> Session {
    session_new(run.clone())
}

/// Start, then submit each answer, scoring and storing it as the window does.
pub fn play(s: &mut VaultSession, run: &Run, answers: &[(Uid, &str)]) {
    let mut session = engine(run);
    apply(s, &mut session, 0, Action::Start);
    for (i, (item, text)) in answers.iter().enumerate() {
        assert_eq!(
            studyfrog_rehearsal::session::current_item(&session),
            Some(*item)
        );
        apply(
            s,
            &mut session,
            1000 * (i as u32 + 1),
            Action::SubmitAnswer((*text).to_owned()),
        );
        let scored: Scored = score_item_answer(s, &Lexical, *item, text).unwrap();
        let scored = Scored {
            engine: "words".into(),
            ..scored
        };
        record_answer_score(s, run.uid, *item, &scored, UnixMs(1_800_000_000_500)).unwrap();
    }
}

/// The run as if its countdown ran out: finished, whatever was left unanswered.
pub fn finish_by_time(s: &mut VaultSession, run: &Run) {
    studyfrog_store::runs::set_run_state(&s.conn, run.uid, RunState::Finished, None).unwrap();
}

pub fn stop(s: &mut VaultSession, run: &Run) {
    let mut session = engine(run);
    apply(s, &mut session, 0, Action::Start);
    apply(s, &mut session, 10, Action::Abort);
}

pub fn run_state(s: &VaultSession, uid: Uid) -> RunState {
    load_run(&s.conn, uid).unwrap().unwrap().state
}
