#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Universal Import Hub UI tests: PDF import through the hub and extension card types.

mod common;
mod import_hub_common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::ext::registry;
use common::flow::{opened, send};
use common::views::harness;
use common::{run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use import_hub_common::{hub, send_ui};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_with_host;
use studyfrog_ui::host::Host;
use studyfrog_ui::import_hub::HubMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{DraftField, Msg};
use studyfrog_ui::state::Selection;

#[test]
fn pdf_import_through_hub_works_for_stacks_and_blocks_unstacked() {
    let (v, mut model, mut worker) = opened("hub-pdf");
    let dir = temp_dir("hub-ui-pdf");
    let pdf = dir.path.join("slides.pdf");
    fs::write(&pdf, pdfmake::three()).unwrap();

    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );
    let mut h = harness(model);
    h.run_steps(2);
    h.get_by_label("Import").click();
    h.run_steps(4);
    send_ui(
        &mut h,
        &mut worker,
        hub(HubMsg::Dropped(pdf.display().to_string())),
    );
    assert!(h.query_by_label("PDF").is_some() && h.query_by_label("Import PDF").is_some());
    h.get_by_label("Unstacked vault notes").click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("PDF files must be placed")
            .is_some()
    );

    h.get_by_label("New stack").click();
    h.run_steps(2);
    for msg in [
        hub(HubMsg::EditNewStack("Physics".into())),
        hub(HubMsg::ImportPdf),
    ] {
        send_ui(&mut h, &mut worker, msg);
    }
    assert!(v.path.join("physics/slides.pdf").is_file());
    assert!(
        h.state()
            .0
            .state
            .pdf_library
            .rows
            .iter()
            .any(|r| r.path == "physics/slides.pdf")
    );
}

#[test]
fn extension_card_types_and_importers_work_in_the_unified_hub() {
    let v = temp_dir("hub-ext");
    fs::create_dir_all(v.path.join("french")).unwrap();
    fs::write(
        v.path.join("french/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\n---\n",
    )
    .unwrap();
    let host = Host {
        registry: registry(),
        ..Host::default()
    };
    let mut worker = worker_with_host(None, None, host);
    let mut model = Model::default();
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::OpenVault(v.path.display().to_string())],
    );
    apply(&mut model, Msg::Select(Selection::Stack("french".into())));
    let mut h = harness(model);
    h.run_steps(2);
    h.get_by_label("Import").click();
    h.run_steps(4);
    for msg in [
        Msg::CardTypeChosen(Some("twice".into())),
        Msg::DraftEdited(DraftField::Front, "salut".into()),
        hub(HubMsg::CreateCardType),
    ] {
        send_ui(&mut h, &mut worker, msg);
    }
    assert_eq!(
        h.state().0.state.vault.as_ref().unwrap().overview.stacks[0].cards,
        2
    );
}
