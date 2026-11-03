#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Proposals for the user's help pages: new pages and edits of pages the model was shown, never others.

use studyfrog_agent::wiki::{Change, parse_proposal, wiki_system};
use studyfrog_agent::wiki_pages::page_id;

#[test]
fn the_instructions_explain_page_changes() {
    let s = wiki_system();
    assert!(s.contains("create_page") && s.contains("update_page"));
}

#[test]
fn new_pages_and_edits_of_shown_user_pages_pass_and_the_rest_is_refused() {
    let shown = vec![page_id("Exam plan"), "A1".to_owned()];
    let reply = r##"{"summary":"pages","changes":[
        {"action":"create_page","name":"Study routine","text":"# Study routine\nDaily."},
        {"action":"update_page","page":"help:Exam plan","text":"# Exam plan\nNew."},
        {"action":"update_page","page":"help:faq","text":"# FAQ\nChanged."},
        {"action":"update_page","page":"faq","text":"x"},
        {"action":"update_page","page":"A1","text":"x"},
        {"action":"create_page","name":"../escape","text":"x"},
        {"action":"create_page","name":".hidden","text":"x"},
        {"action":"create_page","name":"ok","text":""}
    ]}"##;
    let p = parse_proposal(reply, &shown, &[]).unwrap();
    assert_eq!(
        p.changes,
        [
            Change::CreatePage {
                name: "Study routine".into(),
                text: "# Study routine\nDaily.".into()
            },
            Change::UpdatePage {
                name: "Exam plan".into(),
                text: "# Exam plan\nNew.".into()
            },
        ]
    );
    assert_eq!(p.rejected.len(), 6, "{:?}", p.rejected);
}
