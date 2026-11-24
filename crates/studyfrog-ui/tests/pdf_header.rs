#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The PDF viewer header: Previous, Next and More stay in it at 1280 by 800 (three buttons), and More holds the rest.
use studyfrog_ui::effect::EffectVault;

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::{run_to_idle, temp_dir};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_types::{PdfMsg, Side, Zoom};

fn open_with_pdf(label: &str) -> (common::TempDir, Model, Worker) {
    let dir = temp_dir(label);
    fs::create_dir_all(dir.path.join("bio")).unwrap();
    fs::write(dir.path.join("bio/three.pdf"), pdfmake::three()).unwrap();
    let mut model = Model::default();
    let mut worker = worker_new(None);
    let path = dir.path.display().to_string();
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::Vault(EffectVault::OpenVault(path))],
    );
    (dir, model, worker)
}

fn send(model: &mut Model, worker: &mut Worker, m: PdfMsg) {
    let fx = apply(model, Msg::Pdf(m));
    run_to_idle(model, worker, fx);
}

/// The viewer at the 1280 by 800 size the app opens at, with the same frame as the other harnesses.
fn viewer_at_1280(model: Model) -> egui_kittest::Harness<'static, (Model, Vec<Effect>)> {
    use eframe::egui;
    egui_kittest::Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .build_ui_state(
            |ui, (model, effects): &mut (Model, Vec<Effect>)| {
                effects.extend(studyfrog_ui::model::frame(model, ui));
            },
            (model, Vec::new()),
        )
}

/// The labels of the buttons on the header row, the row that holds "Previous", sorted.
fn header_row_buttons(h: &egui_kittest::Harness<'_, (Model, Vec<Effect>)>) -> Vec<String> {
    use eframe::egui::accesskit::Role;
    use egui_kittest::kittest::{NodeT, Queryable};
    let row = h.get_by_label("Previous").rect().min.y;
    let mut labels: Vec<String> = h
        .query_all_by_role(Role::Button)
        .filter(|n| (n.rect().min.y - row).abs() < 4.0)
        .filter_map(|n| n.accesskit_node().label())
        .collect();
    labels.sort();
    labels
}

#[test]
fn the_pdf_header_at_1280_holds_only_the_core_actions_and_more_holds_the_rest() {
    use egui_kittest::kittest::Queryable;
    let mut core = ["More…", "Next", "Previous"].map(str::to_owned);
    core.sort();
    for (i, query) in ["", "Reliab"].into_iter().enumerate() {
        let (_d, mut model, mut worker) = open_with_pdf(&format!("header-core-{i}"));
        send(
            &mut model,
            &mut worker,
            PdfMsg::Open("bio/three.pdf".into(), 0),
        );
        if !query.is_empty() {
            send(&mut model, &mut worker, PdfMsg::Query(query.into()));
        }
        let mut h = viewer_at_1280(model);
        h.run_steps(3);
        assert_eq!(
            header_row_buttons(&h),
            core.to_vec(),
            "the header with query {query:?}"
        );
        for label in ["Page number", "Find in this PDF (Ctrl+F or /)"] {
            assert!(
                h.query_by_label(label).is_some(),
                "{label} stays in the header"
            );
        }
    }
}

#[test]
fn every_action_moved_out_of_the_header_is_still_reachable_from_more() {
    use egui_kittest::kittest::Queryable;
    let (_d, mut model, mut worker) = open_with_pdf("more-reach");
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open("bio/three.pdf".into(), 0),
    );
    send(&mut model, &mut worker, PdfMsg::Query("Reliab".into()));
    let mut h = viewer_at_1280(model);
    h.run_steps(3);
    h.get_by_label("More…").click();
    h.run_steps(2);
    for label in [
        "Back to library",
        "Zoom in",
        "Zoom out",
        "Fit width",
        "Fit page",
        "Previous hit",
        "Next hit",
        "Copy selection",
        "Arrange pages ...",
        "Panels…",
    ] {
        assert!(h.query_by_label(label).is_some(), "{label} is in More");
    }
    h.get_by_label("Zoom in").click();
    h.run_steps(2);
    let zoom = h.state().0.state.pdf.as_ref().unwrap().zoom;
    assert!(zoom != Zoom::FitWidth, "Zoom in works from More");
    h.get_by_label("More…").click();
    h.run_steps(2);
    h.get_by_label("Fit page").click();
    h.run_steps(2);
    let zoom = h.state().0.state.pdf.as_ref().unwrap().zoom;
    assert!(zoom == Zoom::FitPage, "Fit page works from More");
    h.get_by_label("More…").click();
    h.run_steps(2);
    h.get_by_label("Panels…").click();
    h.run_steps(2);
    h.get_by_label("Hide").click();
    h.run_steps(2);
    let side = h.state().0.state.pdf.as_ref().unwrap().side;
    assert!(side == Side::Closed, "Hide works from Panels in More");
}
