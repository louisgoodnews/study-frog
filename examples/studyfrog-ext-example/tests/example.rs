#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The example extension, through the registry like the application uses it.

use proptest::prelude::*;
use studyfrog_ext::registry::{find_card_type, importer_for_file, registry_new};
use studyfrog_ext_example::cloze::expand_cloze;
use studyfrog_ext_example::csv::import_csv;
use studyfrog_ext_example::register;
use studyfrog_ext_example::tsv::import_tsv;

#[test]
fn registering_adds_both_extension_points_once() {
    let mut r = registry_new();
    register(&mut r).unwrap();
    assert_eq!(
        importer_for_file(&r, "deck.tsv").unwrap().id,
        "tab-separated"
    );
    assert_eq!(importer_for_file(&r, "deck.csv").unwrap().id, "csv");
    assert_eq!(
        find_card_type(&r, "cloze").unwrap().name,
        "Cloze (fill in the gaps)"
    );
    assert!(
        register(&mut r).unwrap_err().contains("already registered"),
        "twice is refused"
    );
}

#[test]
fn tab_separated_text_becomes_cards_with_tags_and_skips_headers() {
    let text = "#separator:tab\n#html:true\n\nCapital of France?\tParis\tgeography europe\nLine one<br>two\tAnswer\n";
    let cards = import_tsv(text).unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(
        (cards[0].front.as_str(), cards[0].back.as_str()),
        ("Capital of France?", "Paris")
    );
    assert_eq!(cards[0].tags, ["geography", "europe"]);
    assert_eq!(cards[1].front, "Line one\ntwo");
    assert!(cards[1].tags.is_empty());
    assert!(import_tsv("").unwrap().is_empty());
}

#[test]
fn bug_0011_benchmark_csv_and_prompt_header() {
    let csv = "stack,kind,title,front,back,tags\n\
        psychologie/multivariate-verfahren,card,,Welche Variable wird in Regressionsanalysen als Regressand bezeichnet?,Die abhängige Variable Y.,Regression;Grundbegriffe\n\
        psychologie/multivariate-verfahren,card,,Wie lautet die Gleichung einer zweifachen linearen Regression für die Zufallsvariablen X, Y und Z?,E(Y|X, Z) = beta_0 + beta_1 * X + beta_2 * Z.,Multiple_Regression;Zweifache_Regression\n\
        psychologie/multivariate-verfahren,card,,Was versteht man unter dem Begriff multiple Determiniertheit im Kontext von Regressionen?,Dass eine Variable in der Realität fast nie nur von einer einzigen, sondern von mehreren (meist unbekannten) anderen Variablen beeinflusst wird.,Multiple_Regression;Grundbegriffe\n";
    let cards = import_csv(csv).unwrap();
    assert_eq!(cards.len(), 3);
    assert_eq!(
        cards[0].front,
        "Welche Variable wird in Regressionsanalysen als Regressand bezeichnet?"
    );
    assert_eq!(cards[0].back, "Die abhängige Variable Y.");
    assert_eq!(cards[0].title, "");
    assert_eq!(cards[0].tags, ["Regression", "Grundbegriffe"]);
    assert_eq!(
        cards[1].front,
        "Wie lautet die Gleichung einer zweifachen linearen Regression für die Zufallsvariablen X, Y und Z?"
    );
    assert_eq!(
        cards[1].back,
        "E(Y|X, Z) = beta_0 + beta_1 * X + beta_2 * Z."
    );
    assert_eq!(
        cards[1].tags,
        ["Multiple_Regression", "Zweifache_Regression"]
    );
    assert_eq!(
        cards[2].back,
        "Dass eine Variable in der Realität fast nie nur von einer einzigen, sondern von mehreren (meist unbekannten) anderen Variablen beeinflusst wird."
    );
    assert_eq!(cards[2].tags, ["Multiple_Regression", "Grundbegriffe"]);
}

#[test]
fn bug_0014_unquoted_multiline_and_comma_csv_import() {
    let csv = "stack,kind,title,front,back,tags\n\
        psychologische-diagnostik,card,,Who is the lecturer for the module 'Psychologische Diagnostik und Evaluation'?,Prof. Dr. Christian Kandler,organisation\n\
        psychologische-diagnostik,card,,What is the office address of Prof. Dr. Christian Kandler?,An der Morgenbreede 39, 33615 Bielefeld, Raum H1-115a,organisation\n\
        psychologische-diagnostik,card,,What are the four main reasons why we need accurate psychological diagnostics?,1. Accurate description of human experience and behavior\n\
2. Testing psychological theories explaining human experience and behavior\n\
3. Psychologically grounded prediction and decision making\n\
4. Evaluation of predictions and psychological measures,objectives\n\
        psychologische-diagnostik,card,,What are the three main perspectives of psychological diagnostics mentioned in the slides?,Idiographic, Nomothetic, and Differential,perspectives\n";
    let cards = import_tsv(csv).unwrap();
    assert_eq!(cards.len(), 4);
    assert_eq!(
        cards[1].back,
        "An der Morgenbreede 39, 33615 Bielefeld, Raum H1-115a"
    );
    assert_eq!(cards[1].tags, ["organisation"]);
    assert_eq!(
        cards[2].front,
        "What are the four main reasons why we need accurate psychological diagnostics?"
    );
    assert_eq!(
        cards[2].back,
        "1. Accurate description of human experience and behavior\n\
2. Testing psychological theories explaining human experience and behavior\n\
3. Psychologically grounded prediction and decision making\n\
4. Evaluation of predictions and psychological measures"
    );
    assert_eq!(cards[2].tags, ["objectives"]);
    assert_eq!(cards[3].back, "Idiographic, Nomothetic, and Differential");
    assert_eq!(cards[3].tags, ["perspectives"]);
}

#[test]
fn csv_with_quotes_and_commas_in_content() {
    let csv = "stack,kind,title,front,back,tags\nfrench,card,,\"Quelle est la règle, en général?\",\"C'est la règle \"\"standard\"\".\",grammaire;base\n";
    let cards = import_csv(csv).unwrap();
    assert_eq!(cards[0].front, "Quelle est la règle, en général?");
    assert_eq!(cards[0].back, "C'est la règle \"standard\".");
    assert_eq!(cards[0].tags, ["grammaire", "base"]);
}

#[test]
fn headerless_tsv_and_csv_import() {
    let tsv = "Term 1\tDefinition 1\nTerm 2\tDefinition 2\ttag1 tag2\n";
    let cards = import_tsv(tsv).unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].front, "Term 1");
    assert_eq!(cards[0].tags, Vec::<String>::new());
    assert_eq!(cards[1].tags, ["tag1", "tag2"]);

    let csv = "\"Question 1, with comma\",\"Answer 1\"\n\"Question 2\",\"Answer 2\",\"tag;tag2\"\n";
    let cards = import_csv(csv).unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].front, "Question 1, with comma");
    assert_eq!(cards[1].tags, ["tag", "tag2"]);
}

#[test]
fn cross_format_delimiter_auto_detection() {
    let csv = "front,back,tags\nWhat is 2+2?,4,math\n";
    let cards = import_tsv(csv).unwrap();
    assert_eq!(cards[0].front, "What is 2+2?");

    let tsv = "front\tback\ttags\nWhat is 3+3?\t6\tmath\n";
    let cards = import_csv(tsv).unwrap();
    assert_eq!(cards[0].front, "What is 3+3?");
}

#[test]
fn a_bad_line_is_reported_with_its_number() {
    let e = import_tsv("a\tb\nonly one column\nc\td\n").unwrap_err();
    assert!(e.starts_with("line 2:"), "{e}");
    assert!(
        import_tsv("front\t\n").unwrap_err().contains("line 1"),
        "an empty back is refused"
    );
    assert!(
        import_csv("\"unclosed quote,front,back")
            .unwrap_err()
            .contains("unclosed quote")
    );
    assert!(
        import_csv(",back\n")
            .unwrap_err()
            .contains("front of a card cannot be empty")
    );
    assert!(
        import_csv("stack,kind,title\na,b,c\n")
            .unwrap_err()
            .contains("header must contain front and back columns")
    );
}

#[test]
fn a_cloze_note_becomes_one_card_per_gap() {
    let cards =
        expand_cloze("The {{c1::mitochondria}} is the {{c2::powerhouse::energy}} of the cell")
            .unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].front, "The [...] is the powerhouse of the cell");
    assert_eq!(
        cards[0].back,
        "The **mitochondria** is the powerhouse of the cell"
    );
    assert_eq!(
        cards[1].front, "The mitochondria is the [energy] of the cell",
        "the hint is shown"
    );
    assert_eq!(
        cards[1].back,
        "The mitochondria is the **powerhouse** of the cell"
    );
}

#[test]
fn gaps_with_the_same_number_hide_together() {
    let cards = expand_cloze("{{c1::Paris}} is the capital of {{c1::France}}").unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].front, "[...] is the capital of [...]");
}

#[test]
fn broken_cloze_sources_are_explained() {
    for (src, needle) in [
        ("no gaps here", "no gaps found"),
        ("open {{c1::never closed", "never closed"),
        ("{{c0::x}}", "start at 1"),
        ("{{cx::x}}", "not a gap number"),
        ("{{c1::}}", "no answer"),
        ("{{d1::x}}", "starts with c"),
        ("{{c1 x}}", "needs ::"),
    ] {
        assert!(expand_cloze(src).unwrap_err().contains(needle), "{src}");
    }
}

proptest! {
    #[test]
    fn nothing_panics_and_a_front_never_leaks_its_own_gap(src in "[a-z {}:c0-9]{0,40}") {
        let _ = import_tsv(&src);
        let _ = import_csv(&src);
        if let Ok(cards) = expand_cloze(&src) {
            prop_assert!(!cards.is_empty());
            prop_assert!(cards.iter().all(|c| !c.front.contains("**")));
        }
    }
}
