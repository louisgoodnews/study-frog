#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! The vault, participants, sources, providers and fake model shared by the classroom tests.

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

use studyfrog_agent::lecture_prompt::PromptMessage;
use studyfrog_agent::provider::{Protocol, Provider};
use studyfrog_core::classroom::{Participant, SourceKind, SourceRef};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_material::pdf_source_uid;
use studyfrog_service::classroom_turns::start_session;
use studyfrog_service::create::IdSource;
use studyfrog_service::pdf_export::{ExportTarget, default_pdf_export_options, export_pdf};
use studyfrog_service::pdfs::index_pdfs_fully;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::classroom_store::load_session;
use studyfrog_store::reindex::reindex;

pub const AT: UnixMs = UnixMs(5_000);

pub struct Dir(pub PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A vault with a stack "Bio" (note Cells, uid 3), a note Folien (uid 4) exported as folien.pdf.
pub fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-classroom-{label}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bio")).unwrap();
    fs::write(
        dir.join("bio/_stack.md"),
        "---\nid: 00000000000000000000000002\nkind: stack\ntitle: Bio\n---\n",
    )
    .unwrap();
    fs::write(
        dir.join("bio/cells.md"),
        "---\nid: 00000000000000000000000003\nkind: note\ntitle: Cells\n---\n# Cells\nMitochondria: organelles that produce ATP for the cell.\n",
    )
    .unwrap();
    fs::write(
        dir.join("folien.md"),
        "---\nid: 00000000000000000000000004\nkind: note\ntitle: Folien\n---\n# Zellwand\nThe cell wall gives plant cells their shape.\n",
    )
    .unwrap();
    let (mut s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    reindex(&mut s.conn, &s.root).unwrap();
    export_pdf(
        &s,
        &ExportTarget::SingleItem(Uid(4)),
        &default_pdf_export_options(),
        &dir.join("folien.pdf"),
    )
    .unwrap();
    index_pdfs_fully(&mut s, i64::MAX).unwrap();
    (Dir(dir), s)
}

pub fn people() -> Vec<Participant> {
    vec![
        Participant::Teacher {
            name: "Dr Lin".to_owned(),
        },
        Participant::SimulatedStudent {
            name: "Ada".to_owned(),
        },
        Participant::SimulatedStudent {
            name: "Ben".to_owned(),
        },
        Participant::User {
            name: "Cleo".to_owned(),
        },
    ]
}

/// The teacher, three simulated students (Ada, Ben, Dan) and the user (Cleo), with Dan last. Students
/// split into groups of two as [Ada, Ben] and [Cleo, Dan], so the user's group is group 1.
pub fn four_students() -> Vec<Participant> {
    let mut out = people();
    out.push(Participant::SimulatedStudent {
        name: "Dan".to_owned(),
    });
    out
}

/// Stack Bio, note Folien and the lecture PDF, in that order.
pub fn sources() -> Vec<SourceRef> {
    vec![
        SourceRef {
            kind: SourceKind::Stack,
            uid: Uid(2),
        },
        SourceRef {
            kind: SourceKind::Note,
            uid: Uid(4),
        },
        SourceRef {
            kind: SourceKind::Pdf,
            uid: pdf_source_uid("folien.pdf"),
        },
    ]
}

pub fn start(s: &VaultSession) -> Uid {
    let ids = IdSource {
        now_ms: 1_000,
        random: [3; 10],
    };
    start_session(s, ids, "Cell lecture", &sources(), people()).unwrap()
}

pub fn local() -> Provider {
    Provider {
        name: "local".to_owned(),
        protocol: Protocol::Ollama,
        base_url: "http://localhost:11434".to_owned(),
        model: "qwen2.5:1.5b".to_owned(),
        allowed: false,
    }
}

pub fn remote(allowed: bool) -> Provider {
    Provider {
        name: "cloud".to_owned(),
        protocol: Protocol::OpenAiCompatible,
        base_url: "https://api.example.com/v1".to_owned(),
        model: "some-model".to_owned(),
        allowed,
    }
}

/// A fake model that logs every prompt (system then user) and answers by task: numbered teacher
/// replies, numbered student questions.
pub fn fake(
    log: &RefCell<Vec<String>>,
) -> impl FnMut(&[PromptMessage]) -> Result<String, String> + '_ {
    move |messages| {
        let n = log.borrow().len() + 1;
        let system = &messages[0].content;
        let user = &messages[1].content;
        log.borrow_mut().push(format!("{system}\n{user}"));
        Ok(if system.contains("Task: answer") {
            format!("Teacher answer {n}.")
        } else if system.contains("a student in a study classroom") {
            format!("Question {n}?")
        } else {
            format!("Teacher reply {n}.")
        })
    }
}

pub fn stored(s: &VaultSession, id: Uid) -> studyfrog_core::classroom::ClassroomSession {
    load_session(&s.conn, id).unwrap().unwrap()
}
