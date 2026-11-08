#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! What a run shows for cards and notes with cloze gaps, and what typed answers are compared with (STORY-0108).

mod recall_common;

use recall_common::*;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::compare::{score_item_answer, score_item_answer_with};
use studyfrog_service::content::load_item_content;
use studyfrog_service::create::IdSource;
use studyfrog_service::create_more::{NewItem, NewKind, create_item};
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::score::Verdict;
use studyfrog_store::schedule::insert_review_timed;

const FRONT: &str = "The {{c1::sun}} is a {{c2::star::kind of body}}.";

fn gapped(label: &str) -> Dir {
    let fixed = format!(
        "---\nid: {}\nkind: card\ntitle: Fixed\ncloze_group: 2\n---\n{FRONT}\n???\n\n",
        id(11)
    );
    let note = note(12, "Sky", "Light comes from the {{c1::sun::a star}}.\n");
    vault(
        label,
        &[
            (
                "bio/sun.md",
                card(10, "Sun", FRONT, "Extra: it is 4.6 billion years old."),
            ),
            ("bio/fixed.md", fixed),
            ("bio/sky.md", note),
        ],
    )
}

fn review(s: &studyfrog_service::session::VaultSession, item: u128, at: i64) {
    let r = Review {
        item: Uid(item),
        rating: Rating::Good,
        at: UnixMs(at),
        elapsed_days: 1.0,
    };
    insert_review_timed(&s.conn, &r, None).unwrap();
}

#[test]
fn the_groups_of_a_card_take_turns_with_its_reviews() {
    let dir = gapped("cloze-turns");
    let s = open(&dir);
    let c = load_item_content(&s, Uid(10)).unwrap();
    assert_eq!(c.prompt, "The `[...]` is a star.");
    assert!(
        c.answer
            .contains("**<span style=\"color:#1e8449\">sun</span>**"),
        "{}",
        c.answer
    );
    assert!(
        c.answer.ends_with("Extra: it is 4.6 billion years old."),
        "the back follows"
    );
    assert_eq!(c.expected.as_deref(), Some("sun"));
    review(&s, 10, 1_700_000_000_000);
    let c = load_item_content(&s, Uid(10)).unwrap();
    assert_eq!(c.prompt, "The sun is a `[kind of body]`.");
    assert_eq!(c.expected.as_deref(), Some("star"));
    let right = score_item_answer(&s, &Lexical, Uid(10), "star").unwrap();
    assert_eq!(
        right.score.verdict,
        Verdict::Correct,
        "compared with the gap asked now"
    );
}

#[test]
fn a_fixed_group_and_a_note_with_gaps_are_shown_masked() {
    let dir = gapped("cloze-fixed");
    let s = open(&dir);
    let c = load_item_content(&s, Uid(11)).unwrap();
    assert_eq!(c.prompt, "The sun is a `[kind of body]`.");
    let n = load_item_content(&s, Uid(12)).unwrap();
    assert_eq!(n.prompt, "Light comes from the `[a star]`.");
    assert_eq!(n.expected.as_deref(), Some("sun"));
}

#[test]
fn the_reference_the_run_showed_wins_and_plain_cards_are_unchanged() {
    let dir = gapped("cloze-reference");
    let s = open(&dir);
    let wrong = score_item_answer_with(&s, &Lexical, Uid(10), "sun", Some("star")).unwrap();
    assert_eq!(wrong.score.verdict, Verdict::Incorrect);
    let right = score_item_answer_with(&s, &Lexical, Uid(10), "sun", None).unwrap();
    assert_eq!(right.score.verdict, Verdict::Correct);
}

#[test]
fn a_cloze_card_needs_no_back_and_its_title_gives_nothing_away() {
    let dir = gapped("cloze-create");
    let mut s = open(&dir);
    let item = NewItem {
        kind: NewKind::Card,
        title: "",
        text: FRONT,
        answer: "",
        tags: &[],
        parent: Some("bio"),
        source: None,
        priority: None,
        difficulty: None,
        also_in: None,
    };
    let ids = IdSource {
        now_ms: 1_700_000_000_000,
        random: [3; 10],
    };
    let (_, made) = create_item(&mut s, &item, ids).unwrap();
    assert_eq!(made.title, "The [...] is a [...].");
    let text = read(&dir, &made.path);
    assert!(
        text.contains(FRONT),
        "the syntax is kept as written: {text}"
    );
    let plain = NewItem {
        text: "no gaps",
        ..item
    };
    assert!(
        create_item(&mut s, &plain, ids).is_err(),
        "a plain card still needs a back"
    );
}
