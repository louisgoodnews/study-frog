#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Image occlusion cards in a run, their typed answers, and making one from a picture (STORY-0109).

mod recall_common;

use recall_common::*;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::occlusion::{OcclusionMode, occlusion_of};
use studyfrog_md::parse::parse_doc;
use studyfrog_service::compare::score_item_answer;
use studyfrog_service::content::load_item_content;
use studyfrog_service::content_occlusion::{DEFAULT_PROMPT, create_occlusion_card};
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::score::Verdict;
use studyfrog_store::schedule::insert_review_timed;

fn brain(label: &str) -> Dir {
    let card = format!(
        "---\nid: {}\nkind: card\ntitle: Brain\nocclusion:\n  image: \"assets/brain.png\"\n  mode: hide_all\n  masks:\n    - \"0.1000 0.1000 0.2000 0.1000 Hippocampus\"\n    - \"0.5000 0.5000 0.2000 0.1000 Thalamus\"\n---\n![brain](assets/brain.png)\n???\nBoth sit deep in the brain.\n",
        id(10)
    );
    vault(label, &[("bio/brain.md", card)])
}

#[test]
fn the_masks_take_turns_and_the_label_is_the_answer() {
    let dir = brain("occ-turns");
    let s = open(&dir);
    let c = load_item_content(&s, Uid(10)).unwrap();
    assert_eq!(
        c.prompt, DEFAULT_PROMPT,
        "the picture is drawn with its masks, not in the text"
    );
    assert_eq!(c.answer, "**Hippocampus**\n\nBoth sit deep in the brain.");
    assert_eq!(c.expected.as_deref(), Some("Hippocampus"));
    let v = c.occlusion.unwrap();
    assert_eq!(
        (v.image.as_str(), v.mode, v.masks.len(), v.active),
        ("assets/brain.png", OcclusionMode::HideAll, 2, 0)
    );
    let r = Review {
        item: Uid(10),
        rating: Rating::Good,
        at: UnixMs(1_700_000_000_000),
        elapsed_days: 1.0,
    };
    insert_review_timed(&s.conn, &r, None).unwrap();
    let c = load_item_content(&s, Uid(10)).unwrap();
    assert_eq!(c.occlusion.unwrap().active, 1);
    assert_eq!(c.expected.as_deref(), Some("Thalamus"));
    let typed = score_item_answer(&s, &Lexical, Uid(10), "thalamus").unwrap();
    assert_eq!(
        typed.score.verdict,
        Verdict::Correct,
        "compared with the mask asked now"
    );
}

#[test]
fn a_picture_becomes_a_card_without_masks_and_the_picture_is_untouched() {
    let dir = brain("occ-create");
    std::fs::create_dir_all(dir.0.join("assets")).unwrap();
    std::fs::write(dir.0.join("assets/heart.png"), b"not really a png").unwrap();
    let mut s = open(&dir);
    let (_, made) =
        create_occlusion_card(&mut s, "bio", "assets/heart.png", ids(1_700_000_000_000)).unwrap();
    assert_eq!(made.title, "Occlusion: heart");
    let text = read(&dir, &made.path);
    let doc = parse_doc(&text).unwrap();
    let o = occlusion_of(&doc).unwrap();
    assert_eq!((o.image.as_str(), o.masks.len()), ("assets/heart.png", 0));
    assert!(text.contains("![heart](assets/heart.png)"), "{text}");
    assert_eq!(
        std::fs::read(dir.0.join("assets/heart.png")).unwrap(),
        b"not really a png"
    );
    let c = load_item_content(&s, made.uid.unwrap()).unwrap();
    assert!(c.occlusion.is_none(), "no masks yet: shown as a plain card");
    assert!(create_occlusion_card(&mut s, "", "assets/heart.png", ids(1)).is_err());
}
