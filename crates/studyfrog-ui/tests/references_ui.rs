#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless acceptance tests for TASK-0213: References in the interface.

mod common;

use common::views::{assert_all_named, harness, open_model, stack};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::bulk::BulkEdit;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::library_bulk::{BulkMsg, LibraryEffect};
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::modal::ModalMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::{EditAction, Msg};

const CARD_WITH_REFS: &str = "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Vocab\nreferenced_by: [00000000000000000000000002]\n---\nPrompt\n???\nAnswer\n";

#[test]
fn the_create_dialog_offers_an_also_in_choice_and_passes_it_to_create_effect() {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
    let mut h = harness(m);
    h.run_steps(4);
    assert_all_named(&h, "create dialog with also_in");
    assert!(h.query_by_label("Also in (optional)").is_some());

    // Selecting another stack sets also_in
    let fx = apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::SetAlsoIn(Some("maths".into()))),
    );
    assert!(fx.is_empty());
    assert_eq!(
        h.state().0.state.modal.as_ref().and_then(|m| match m {
            studyfrog_ui::modal::Modal::Create(f) => f.also_in.clone(),
            _ => None,
        }),
        Some("maths".into())
    );

    // Submitting produces CreateItem with also_in: Some("maths")
    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::Edit(
            studyfrog_ui::modal::Field::Text,
            "Question?".into(),
        )),
    );
    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::Edit(
            studyfrog_ui::modal::Field::Answer,
            "Answer".into(),
        )),
    );
    let fx = apply(&mut h.state_mut().0, Msg::Modal(ModalMsg::Submit));
    assert!(matches!(
        &fx[..],
        [Effect::CreateItem {
            also_in: Some(s),
            ..
        }] if s == "maths"
    ));
}

#[test]
fn the_item_page_shows_primary_and_referenced_stacks_with_add_and_remove() {
    let mut m = open_model();
    // Add stack 2 to overview
    if let Some(v) = &mut m.state.vault {
        let mut german = stack("German", "german", 1, 1);
        german.uid = Uid(2);
        v.overview.stacks.push(german);
    }
    let doc = parse_doc(CARD_WITH_REFS).unwrap();
    apply(
        &mut m,
        Msg::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "french/vocab.md".into(),
                doc,
                hash: 1,
            }),
        ),
    );

    let mut h = harness(m);
    h.run_steps(4);
    h.get_by_label("Relations").click();
    h.run_steps(2);
    assert!(h.query_by_label("Stacks & references").is_some());
    assert!(h.query_by_label("Remove from German").is_some());
    assert!(h.query_by_label("Add to stack…").is_some());

    // Remove reference action
    apply(
        &mut h.state_mut().0,
        Msg::Edit(EditAction::RemoveReference(
            "00000000000000000000000002".into(),
        )),
    );
    assert!(
        h.state()
            .0
            .state
            .editor
            .as_ref()
            .unwrap()
            .form
            .referenced_by
            .is_empty()
    );

    // Add reference action
    apply(
        &mut h.state_mut().0,
        Msg::Edit(EditAction::AddReference(
            "00000000000000000000000002".into(),
        )),
    );
    assert_eq!(
        h.state()
            .0
            .state
            .editor
            .as_ref()
            .unwrap()
            .form
            .referenced_by,
        vec!["00000000000000000000000002"]
    );
}

#[test]
fn stack_backlog_includes_items_that_are_referenced() {
    use common::views::lib_item;
    use studyfrog_service::fam_refs::RefView;
    use studyfrog_ui::library_ui::LibraryState;
    use studyfrog_ui::stack_backlog::backlog;

    let mut item = lib_item(10, "Chat");
    item.row.due_day = Some(20_000);
    item.references = vec![
        RefView {
            stack: Uid(1),
            folder: Some("french".into()),
            title: "French".into(),
            primary: true,
            value: None,
            percent: None,
            difficulty: None,
            stability: None,
            due_ms: None,
            reviews: 0,
        },
        RefView {
            stack: Uid(2),
            folder: Some("german".into()),
            title: "German".into(),
            primary: false,
            value: None,
            percent: None,
            difficulty: None,
            stability: None,
            due_ms: None,
            reviews: 0,
        },
    ];
    let lib = LibraryState {
        items: vec![item],
        shown: vec![0],
        ..Default::default()
    };
    let b = backlog(&lib, "german", 20_000);
    assert_eq!(b.up_next, vec![0]);
}

#[test]
fn the_library_bulk_bar_supports_add_to_stack() {
    let mut m = open_model();
    m.state.library.selected.insert(Uid(10));
    let fx = apply(
        &mut m,
        Msg::Library(LibraryMsg::Bulk(BulkMsg::Apply(BulkEdit::AddTo(
            "german".into(),
        )))),
    );
    assert_eq!(
        fx,
        [Effect::Library(LibraryEffect::Bulk(
            vec![Uid(10)],
            BulkEdit::AddTo("german".into()),
        ))]
    );
}
