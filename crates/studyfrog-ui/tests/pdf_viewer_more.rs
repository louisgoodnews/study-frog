#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The PDF viewer end to end: the update, the worker with a real file, and the selection helpers.
use common::{run_to_idle, temp_dir};
use std::fs;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectVault;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_types::{PdfEffect, PdfMsg};

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

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

#[test]
fn files_outside_the_vault_or_not_pdfs_or_missing_are_refused_with_a_sentence() {
    let (_d, mut model, mut worker) = open_with_pdf("refuse");
    for bad in [
        "../x.pdf",
        "/etc/passwd",
        "bio/three.txt",
        "bio/none.pdf",
        "a//b.pdf",
    ] {
        send(&mut model, &mut worker, PdfMsg::Open(bad.into(), 0));
        let view = model.state.pdf.as_ref().unwrap();
        assert!(
            view.error.as_deref().is_some_and(|e| e.len() > 10),
            "{bad}: {:?}",
            view.error
        );
        assert!(view.info.is_none());
    }
}

#[test]
fn the_viewer_screen_shows_its_controls_and_asks_for_the_page_picture() {
    use common::views::harness;
    use egui_kittest::kittest::Queryable;
    let (_d, mut model, mut worker) = open_with_pdf("screen");
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open("bio/three.pdf".into(), 0),
    );
    let mut h = harness(model);
    h.run_steps(3);
    for label in ["Previous", "Next", "More…"] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    h.get_by_label("More…").click();
    h.run_steps(2);
    assert!(
        h.query_by_label("Fit width").is_some(),
        "the fit presets are in More"
    );
    h.get_by_label("Panels…").click();
    h.run_steps(2);
    for label in ["Outline", "Pages", "Hide"] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    h.key_press(eframe::egui::Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label("Find in this PDF (Ctrl+F or /)").is_some());
    let asked = h
        .state()
        .1
        .iter()
        .any(|e| matches!(e, Effect::Pdf(PdfEffect::Render { .. })));
    assert!(asked, "the first frame asks for the page picture");
}

#[test]
fn ctrl_f_in_the_viewer_focuses_find_in_this_pdf_and_slash_does_too() {
    use common::views::harness;
    use eframe::egui;
    let (_d, mut model, mut worker) = open_with_pdf("find-shortcut");
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open("bio/three.pdf".into(), 0),
    );
    let mut h = harness(model);
    h.run_steps(3);
    assert!(
        h.ctx.memory(|m| m.focused()).is_none(),
        "nothing has the keyboard yet"
    );
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::F);
    h.run_steps(2);
    assert!(
        h.ctx.memory(|m| m.focused()).is_some(),
        "Ctrl+F focuses the find field"
    );
    h.ctx
        .memory_mut(|m| m.surrender_focus(m.focused().unwrap()));
    h.run_steps(2);
    h.key_press(egui::Key::Slash);
    h.run_steps(2);
    assert!(h.ctx.memory(|m| m.focused()).is_some(), "/ focuses it too");
}
