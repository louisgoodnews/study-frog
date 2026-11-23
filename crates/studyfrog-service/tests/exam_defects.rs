#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Mock exam from notes (STORY-0186, BUG-0052, BUG-0053): a formula note makes a calculation question, and a card's
//! reference answer is its answer alone.

use std::fs;
use std::path::PathBuf;

use studyfrog_rehearsal::curriculum_build::{assemble, plan, templates};
use studyfrog_service::exam_save::notes_chunks;
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::reindex::reindex;

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-exam-defects-{label}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("stat")).unwrap();
    fs::write(
        dir.join("stat/_stack.md"),
        "---\nid: 00000000000000000000000002\nkind: stack\ntitle: Statistik\n---\n",
    )
    .unwrap();
    fs::write(
        dir.join("stat/formel.md"),
        "---\nid: 00000000000000000000000003\nkind: note\ntitle: Formel\n---\nStandardfehler = s / Wurzel(n) = 2 / 2 = 1\n",
    )
    .unwrap();
    fs::write(
        dir.join("stat/mittel.md"),
        "---\nid: 00000000000000000000000004\nkind: card\ntitle: Mittelwert\n---\nWas ist der Mittelwert?\n???\nSumme durch Anzahl\n",
    )
    .unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

#[test]
fn a_formula_note_gives_a_calculation_part_and_the_prompt_does_not_print_the_formula() {
    let (_d, mut s) = vault("formula");
    reindex(&mut s.conn, &s.root).unwrap();
    let chunks = notes_chunks(&s, Some("stat")).unwrap();
    let (exam, keys) = assemble("Stat", &templates(&chunks, plan(10)));
    let part_c = exam
        .sections
        .iter()
        .find(|sec| sec.name == "Part C: calculations")
        .expect("a calculation part");
    assert_eq!(part_c.questions.len(), 1);
    assert!(!part_c.questions[0].prompt.contains("Wurzel"));
    let key = keys
        .iter()
        .find(|k| k.prompt.starts_with("Give the formula"))
        .expect("a key for the formula");
    assert_eq!(key.rubric, ["Standardfehler", "s / Wurzel(n)"]);
}

#[test]
fn a_cards_reference_answer_is_its_answer_alone() {
    let (_d, mut s) = vault("card");
    reindex(&mut s.conn, &s.root).unwrap();
    let chunks = notes_chunks(&s, Some("stat")).unwrap();
    let (_, keys) = assemble("Stat", &templates(&chunks, plan(10)));
    let card = keys
        .iter()
        .find(|k| k.sample_answer.contains("Summe durch Anzahl"))
        .expect("a key for the card");
    assert_eq!(card.sample_answer, "Summe durch Anzahl");
    assert!(
        keys.iter()
            .all(|k| !k.sample_answer.contains("Was ist der Mittelwert"))
    );
    assert!(
        keys.iter()
            .all(|k| !k.rubric.iter().any(|r| r.contains("Was ist")))
    );
}
