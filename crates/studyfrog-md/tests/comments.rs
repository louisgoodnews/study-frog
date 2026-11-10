//! Test comments roundtrip in markdown front-matter.

use studyfrog_core::comment::new_comment;
use studyfrog_md::comments::{comments_of, with_comments};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;

#[test]
fn doc_comments_roundtrip() {
    let input = r#"---
id: 01H7V0KXYZ0000000000000001
kind: card
title: Mitochondria
---
What is the powerhouse of the cell?
???
Mitochondria
"#;
    let doc = parse_doc(input).expect("parse valid doc");
    assert!(comments_of(&doc).is_empty());

    let c1 = new_comment(
        "c_1",
        "user",
        1731140000000,
        "Remember to check cell structure",
    );
    let c2 = new_comment(
        "c_2",
        "agent",
        1731140050000,
        "Mitochondria produce ATP via oxidative phosphorylation",
    );

    let updated = with_comments(&doc, &[c1.clone(), c2.clone()]);
    let comments = comments_of(&updated);
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0], c1);
    assert_eq!(comments[1], c2);

    let rendered = render_doc(&updated).expect("render updated doc");
    assert!(rendered.contains("comments:"));
    assert!(rendered.contains("c_1"));
    assert!(rendered.contains("oxidative phosphorylation"));

    let parsed_back = parse_doc(&rendered).expect("parse rendered doc");
    let parsed_comments = comments_of(&parsed_back);
    assert_eq!(parsed_comments.len(), 2);
    assert_eq!(parsed_comments[0], c1);
    assert_eq!(parsed_comments[1], c2);

    // Test removing comments
    let cleared = with_comments(&parsed_back, &[]);
    assert!(comments_of(&cleared).is_empty());
    let rendered_cleared = render_doc(&cleared).expect("render cleared doc");
    assert!(!rendered_cleared.contains("comments:"));
}
