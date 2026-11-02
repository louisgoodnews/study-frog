#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The library list's logic: loading, the stack and the search, sorting and the columns.

mod common;

use common::views::{lib_item, open_model};
use studyfrog_core::query::{Field, Order};
use studyfrog_core::status::Status;
use studyfrog_service::library_rows::LibraryItem;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::library_ui::{
    COLUMNS, Column, DEFAULT_COLUMNS, LibraryMsg, cell, columns_from_text, columns_to_text,
};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection};

fn item(n: u128, title: &str, stacks: &[&str]) -> LibraryItem {
    let mut i = lib_item(n, title);
    i.row.stacks = stacks.iter().map(|s| (*s).to_owned()).collect();
    i
}

fn four() -> Vec<LibraryItem> {
    let mut b = item(2, "Etre", &["french", "French", "french/verbs", "Verbs"]);
    b.row.status = Status::Done;
    b.row.familiarity = Some(0.756);
    let mut c = item(3, "Primes", &["maths", "Mathematics"]);
    c.row.reviews = 7;
    vec![
        item(1, "Bonjour", &["french", "French"]),
        b,
        c,
        item(4, "Loose", &[]),
    ]
}

fn loaded() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(four()))));
    m
}

fn shown(m: &Model) -> Vec<String> {
    m.state
        .library
        .shown
        .iter()
        .map(|i| m.state.library.items[*i].row.title.clone())
        .collect()
}

#[test]
fn the_rows_are_asked_for_when_the_library_opens_and_after_changes() {
    let mut m = Model::default();
    let fx = apply(&mut m, Msg::VaultOpened(Ok(common::views::info())));
    assert!(
        !fx.contains(&Effect::LoadLibrary),
        "the library is not on screen yet"
    );
    let fx = apply(&mut m, Msg::Navigate(Screen::Library));
    assert!(
        fx.contains(&Effect::LoadLibrary),
        "opening the library reads the rows"
    );
    assert!(
        !apply(&mut m, Msg::Tick).contains(&Effect::LoadLibrary),
        "and only once"
    );
    let fx = apply(&mut m, Msg::ItemsLoaded(Ok(Vec::new())));
    assert!(
        fx.contains(&Effect::LoadLibrary),
        "a change of the items reads them again"
    );
}

#[test]
fn the_stack_decides_what_is_listed_and_a_stack_includes_the_stacks_below_it() {
    let mut m = loaded();
    assert_eq!(
        shown(&m),
        ["Bonjour", "Etre", "Primes", "Loose"],
        "all items, in path order"
    );
    assert_eq!(m.state.library.scope_total, 4);
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    assert_eq!(
        shown(&m),
        ["Bonjour", "Etre"],
        "the stack and the one below it"
    );
    apply(&mut m, Msg::Select(Selection::Stack("french/verbs".into())));
    assert_eq!(shown(&m), ["Etre"]);
    apply(&mut m, Msg::Select(Selection::Inbox));
    assert_eq!(shown(&m), ["Loose"]);
    assert_eq!(m.state.library.scope_total, 1);
}

#[test]
fn the_search_narrows_the_list_by_title_and_the_count_says_how_many_of_how_many() {
    let mut m = loaded();
    apply(&mut m, Msg::SearchEdited("PRIM".into()));
    assert_eq!(shown(&m), ["Primes"]);
    assert_eq!(
        (m.state.library.shown.len(), m.state.library.scope_total),
        (1, 4)
    );
    apply(&mut m, Msg::SearchEdited("zzz".into()));
    assert!(shown(&m).is_empty());
    apply(&mut m, Msg::SearchEdited(String::new()));
    assert_eq!(shown(&m).len(), 4);
}

#[test]
fn a_heading_sorts_a_second_click_reverses_and_shift_adds_a_key() {
    let mut m = loaded();
    apply(&mut m, Msg::Library(LibraryMsg::Sort(Column::Title, false)));
    assert_eq!(shown(&m), ["Bonjour", "Etre", "Loose", "Primes"]);
    apply(&mut m, Msg::Library(LibraryMsg::Sort(Column::Title, false)));
    assert_eq!(shown(&m), ["Primes", "Loose", "Etre", "Bonjour"]);
    apply(
        &mut m,
        Msg::Library(LibraryMsg::Sort(Column::Reviews, false)),
    );
    assert_eq!(
        shown(&m)[3],
        "Primes",
        "ascending puts the item with the most reviews last"
    );
}

#[test]
fn sorting_by_several_columns_keeps_at_most_three_keys_and_ignores_unsortable_ones() {
    let mut m = loaded();
    for c in [Column::Status, Column::Reviews, Column::Title, Column::Type] {
        apply(&mut m, Msg::Library(LibraryMsg::Sort(c, true)));
    }
    let keys: Vec<Field> = m.state.library.sort.iter().map(|o| o.field).collect();
    assert_eq!(keys.len(), 3, "{keys:?}");
    let before = m.state.library.sort.clone();
    apply(&mut m, Msg::Library(LibraryMsg::Sort(Column::Tags, false)));
    assert_eq!(m.state.library.sort, before, "tags cannot be sorted");
    apply(
        &mut m,
        Msg::Library(LibraryMsg::Sort(Column::Familiarity, false)),
    );
    assert_eq!(
        m.state.library.sort,
        [Order {
            field: Field::Familiarity,
            descending: false
        }]
    );
}

#[test]
fn columns_are_shown_or_hidden_in_a_fixed_order_the_title_stays_and_the_choice_is_saved() {
    let mut m = loaded();
    assert_eq!(m.state.library.columns, DEFAULT_COLUMNS);
    let fx = apply(
        &mut m,
        Msg::Library(LibraryMsg::ToggleColumn(Column::Familiarity)),
    );
    assert!(m.state.library.columns.contains(&Column::Familiarity));
    let [Effect::SaveAppSettings(app)] = &fx[..] else {
        panic!("{fx:?}")
    };
    assert_eq!(
        app.library_columns.as_deref(),
        Some("title,type,status,priority,familiarity,due")
    );
    apply(
        &mut m,
        Msg::Library(LibraryMsg::ToggleColumn(Column::Status)),
    );
    assert!(!m.state.library.columns.contains(&Column::Status));
    assert!(
        apply(
            &mut m,
            Msg::Library(LibraryMsg::ToggleColumn(Column::Title))
        )
        .is_empty(),
        "the title cannot be hidden"
    );
    assert_eq!(m.state.library.columns[0], Column::Title);
}

#[test]
fn stored_columns_read_back_ignoring_unknown_words_and_starting_with_the_title() {
    assert_eq!(
        columns_from_text("status,nonsense,due,status"),
        [Column::Title, Column::Status, Column::Due]
    );
    assert_eq!(columns_from_text(""), [Column::Title]);
    for (c, word, _) in COLUMNS {
        assert!(columns_from_text(word).contains(&c), "{word}");
    }
    let all: Vec<Column> = COLUMNS.iter().map(|(c, _, _)| *c).collect();
    assert_eq!(columns_from_text(&columns_to_text(&all)), all);
    let mut m = Model::default();
    let app = studyfrog_core::settings::AppSettings {
        library_columns: Some("title,path".into()),
        ..Default::default()
    };
    apply(&mut m, Msg::Started { app, error: None });
    assert_eq!(m.state.library.columns, [Column::Title, Column::Path]);
}

#[test]
fn cells_show_what_a_row_says_in_words() {
    let list = four();
    let (b, c) = (&list[1].row, &list[2].row);
    assert_eq!(cell(b, Column::Status), "Complete");
    assert_eq!(cell(b, Column::Familiarity), "76 %");
    assert_eq!(cell(c, Column::Familiarity), "–");
    assert_eq!(cell(b, Column::Stack), "Verbs");
    assert_eq!(cell(&list[3].row, Column::Stack), "–");
    assert_eq!(cell(c, Column::Reviews), "7");
    assert_eq!(cell(c, Column::Type), "card");
    assert_eq!(cell(c, Column::Priority), "medium");
    assert_eq!(cell(c, Column::Due), "–");
    assert_eq!(cell(c, Column::Tags), "–");
    assert_eq!(cell(c, Column::Path), "french/3.md");
}
