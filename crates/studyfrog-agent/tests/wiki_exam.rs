#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Checking what the model proposes and the exam verdicts.

use studyfrog_agent::exam::{Verdict, parse_results, without_results};
use studyfrog_agent::wiki::{Change, MAX_CHANGES, parse_proposal};

fn uids() -> Vec<String> {
    vec!["A1".into(), "B2".into()]
}

fn stacks() -> Vec<String> {
    vec!["french".into(), "french/verbs".into()]
}

#[test]
fn a_good_proposal_is_read_even_inside_a_fence_and_chatter() {
    let reply = "Sure!\n```json\n{\"summary\":\"Added an index\",\"changes\":[\
        {\"action\":\"create_note\",\"stack\":\"french\",\"title\":\"Index\",\"body\":\"- [[Verbs]]\"},\
        {\"action\":\"create_card\",\"stack\":\"french/verbs\",\"front\":\"être?\",\"back\":\"to be\"},\
        {\"action\":\"create_question\",\"stack\":\"french\",\"prompt\":\"Plural of cheval?\",\"reference\":\"chevaux\"},\
        {\"action\":\"update_item\",\"uid\":\"A1\",\"fields\":{\"body\":\"new\",\"colour\":\"red\"}}]}\n```\nDone.";
    let p = parse_proposal(reply, &uids(), &stacks()).unwrap();
    assert_eq!(p.summary, "Added an index");
    assert_eq!(p.changes.len(), 4);
    assert!(p.rejected.is_empty());
    assert!(
        matches!(&p.changes[3], Change::UpdateItem { uid, fields } if uid == "A1" && fields == &[("body".to_owned(), "new".to_owned())]),
        "unknown fields are dropped"
    );
}

#[test]
fn unsafe_or_unknown_changes_are_left_out_with_a_reason() {
    let reply = r#"{"summary":"x","changes":[
        {"action":"create_note","stack":"../outside","title":"T","body":"b"},
        {"action":"create_note","stack":"french","title":"two\nlines","body":"b"},
        {"action":"create_card","stack":"french","front":"","back":"b"},
        {"action":"update_item","uid":"ZZ","fields":{"body":"b"}},
        {"action":"update_item","uid":"A1","fields":{"colour":"b"}},
        {"action":"delete_everything"},
        {"action":"create_note","stack":"french","title":"ok","body":"fine"}]}"#;
    let p = parse_proposal(reply, &uids(), &stacks()).unwrap();
    assert_eq!(p.changes.len(), 1);
    assert_eq!(p.rejected.len(), 6, "{:?}", p.rejected);
    assert!(
        p.rejected[0].contains("not one of the listed folders")
            && p.rejected[3].contains("not one of the items")
    );
}

#[test]
fn nonsense_is_an_error_and_a_long_list_is_cut() {
    assert!(parse_proposal("I cannot do that", &uids(), &stacks()).is_err());
    assert!(parse_proposal("{\"summary\":\"x\"}", &uids(), &stacks()).is_err());
    let many: Vec<String> = (0..60).map(|i| format!("{{\"action\":\"create_note\",\"stack\":\"french\",\"title\":\"N{i}\",\"body\":\"b\"}}")).collect();
    let p = parse_proposal(
        &format!("{{\"summary\":\"s\",\"changes\":[{}]}}", many.join(",")),
        &uids(),
        &stacks(),
    )
    .unwrap();
    assert_eq!(p.changes.len(), MAX_CHANGES);
    assert!(p.rejected[0].contains("left out"));
}

#[test]
fn exam_verdicts_are_read_only_for_known_items_and_only_at_the_end() {
    let reply = "Well done!\n{\"results\": [{\"id\": \"A1\", \"verdict\": \"correct\"}, {\"id\": \"B2\", \"verdict\": \"Partly\"}, {\"id\": \"Q9\", \"verdict\": \"correct\"}, {\"id\": \"A1\", \"verdict\": \"wrong\"}]}";
    let r = parse_results(reply, &uids()).unwrap();
    assert_eq!(
        r.iter()
            .map(|x| (x.uid.as_str(), x.verdict))
            .collect::<Vec<_>>(),
        [("A1", Verdict::Correct), ("B2", Verdict::Partial)]
    );
    assert_eq!(without_results(reply), "Well done!");
    assert!(parse_results("Next question: ...", &uids()).is_none());
    assert!(
        parse_results(
            "{\"results\": [{\"id\": \"A1\", \"verdict\": \"meh\"}]}",
            &uids()
        )
        .is_none()
    );
    assert_eq!(without_results("just text"), "just text");
}
