#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Temporary vaults and a loop that runs effects until the UI is idle.

pub mod views;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, execute};
use studyfrog_ui::model::{Model, apply};

static COUNTER: AtomicU32 = AtomicU32::new(0);
pub const NOW_NS: i64 = i64::MAX / 2;

pub struct TempDir {
    pub path: PathBuf,
}

pub fn temp_dir(label: &str) -> TempDir {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("studyfrog-ui-{label}-{}-{n}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    TempDir { path }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// A vault with a "French" stack (two cards, one note) and one loose note.
pub fn sample_vault(label: &str) -> TempDir {
    let dir = temp_dir(label);
    let id = |n: u128| format_ulid(Uid(n));
    let write = |rel: &str, text: String| {
        let p = dir.path.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    };
    write(
        "french/_stack.md",
        format!("---\nid: {}\nkind: stack\ntitle: French\n---\n", id(1)),
    );
    write(
        "french/a.md",
        format!(
            "---\nid: {}\nkind: card\ntitle: A\n---\nq\n???\na\n",
            id(10)
        ),
    );
    write(
        "french/b.md",
        format!(
            "---\nid: {}\nkind: card\ntitle: B\n---\nq\n???\na\n",
            id(11)
        ),
    );
    write(
        "french/c.md",
        format!("---\nid: {}\nkind: note\ntitle: C\n---\nbody\n", id(12)),
    );
    write(
        "loose.md",
        format!("---\nid: {}\nkind: note\ntitle: Loose\n---\n", id(13)),
    );
    dir
}

/// Execute effects, apply the resulting messages, and repeat until nothing is left to do.
pub fn run_to_idle(model: &mut Model, worker: &mut Worker, mut effects: Vec<Effect>) {
    for _ in 0..20 {
        if effects.is_empty() {
            return;
        }
        let msgs: Vec<_> = effects
            .drain(..)
            .flat_map(|e| execute(worker, e, NOW_NS))
            .collect();
        effects = msgs.into_iter().flat_map(|m| apply(model, m)).collect();
    }
    panic!("effects did not settle");
}
