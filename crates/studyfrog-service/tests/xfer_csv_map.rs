#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0208: CSV with other column names, orders, delimiters and encodings; every problem at once.

use studyfrog_core::plan::ItemKind;
use studyfrog_service::xfer_csv_map::{CsvOptions, Mapping, ai_message, problem_text, read_csv};

fn read(text: &str, o: &CsvOptions) -> studyfrog_service::xfer_csv_map::CsvRead {
    read_csv(text.as_bytes(), o).unwrap()
}

#[test]
fn other_names_and_orders_are_understood_from_the_header() {
    let r = read(
        "Answer;Question;Deck;Tag\nle chat;the cat;french;animals;x\n",
        &CsvOptions::default(),
    );
    assert_eq!(r.sheet.delimiter, ';');
    assert!(r.sheet.header);
    let it = &r.bundle.items[0];
    assert_eq!(
        (it.text.as_str(), it.answer.as_str(), it.stack.as_str()),
        ("the cat", "le chat", "french")
    );
    assert_eq!(it.tags, ["animals"]);
}

#[test]
fn a_file_with_two_columns_and_no_header_is_front_and_back() {
    let r = read(
        "the cat,le chat\nthe dog,le chien\n",
        &CsvOptions::default(),
    );
    assert!(!r.sheet.header);
    assert_eq!(r.bundle.items.len(), 2);
    assert_eq!(r.sheet.names, ["Column 1", "Column 2"]);
    assert_eq!(r.bundle.items[1].answer, "le chien");
}

#[test]
fn the_mapping_the_user_chose_wins_and_a_kind_column_makes_notes_and_questions() {
    let text = "a,b,c,d\nNote body,,note,Heading\nWhy?,Because,question,\n";
    let map = Mapping {
        front: 0,
        back: 1,
        title: Some(3),
        tags: None,
        stack: None,
        kind: Some(2),
    };
    let o = CsvOptions {
        header: Some(true),
        map: Some(map),
        ..CsvOptions::default()
    };
    let r = read(text, &o);
    let kinds: Vec<_> = r.bundle.items.iter().map(|i| i.kind).collect();
    assert_eq!(kinds, [ItemKind::Note, ItemKind::Question]);
    assert_eq!(r.bundle.items[0].title, "Heading");
    assert!(r.problems.is_empty());
}

#[test]
fn all_problems_are_reported_at_once_with_row_and_column() {
    let text = "front,back,kind\nok,fine,card\n,missing front,card\nno back,,card\nx,y,essay\n";
    let r = read(text, &CsvOptions::default());
    assert_eq!(r.bundle.items.len(), 1, "only the good row is kept");
    let rows: Vec<(usize, &str)> = r
        .problems
        .iter()
        .map(|p| (p.row, p.column.as_str()))
        .collect();
    assert_eq!(rows, [(3, "front"), (4, "back"), (5, "kind")]);
    assert!(problem_text(&r.problems[2]).contains("card, question or note"));
    assert_eq!(r.bundle.warnings.len(), 3);
    let message = ai_message("LAYOUT", &r.problems);
    assert!(
        message.contains("LAYOUT")
            && message.contains("row 3, column front")
            && message.contains("row 5")
    );
}

#[test]
fn the_delimiter_and_the_encoding_are_detected_and_can_be_forced() {
    let latin1 = b"front\tback\ncaf\xe9\tcoffee\n";
    let r = read_csv(latin1, &CsvOptions::default()).unwrap();
    assert_eq!((r.sheet.delimiter, r.sheet.latin1), ('\t', true));
    assert_eq!(r.bundle.items[0].text, "café");
    let bom = "\u{feff}front,back\na,b\n";
    assert_eq!(read(bom, &CsvOptions::default()).bundle.items.len(), 1);
    let forced = CsvOptions {
        delimiter: Some('|'),
        ..CsvOptions::default()
    };
    let r = read("front|back\na,b|c\n", &forced);
    assert_eq!(r.bundle.items[0].text, "a,b");
}

#[test]
fn nothing_usable_is_an_error_not_a_crash() {
    assert!(read_csv(b"", &CsvOptions::default()).is_err());
    assert!(read_csv(b"single\nvalues\n", &CsvOptions::default()).is_err());
    let _ = read_csv(&[0xff, 0x00, 0x22], &CsvOptions::default());
}
