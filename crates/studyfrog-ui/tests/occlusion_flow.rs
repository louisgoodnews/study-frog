#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! An occlusion card made from a picture (as the PDF viewer's saved pictures offer) through the real worker,
//! opened on its page, its masks saved to the file (STORY-0109).

mod common;

use common::flow::send;
use common::views::open_model;
use common::{run_to_idle, temp_dir};
use studyfrog_core::uid::Uid;
use studyfrog_md::occlusion::{Mask, occlusion_of};
use studyfrog_md::ulid::format_ulid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::{EditAction, Msg};
use studyfrog_ui::recall_types::{RecallEffect, RecallMsg};
use studyfrog_ui::state::Screen;

#[test]
fn the_stack_must_be_chosen_first() {
    let mut m = open_model();
    let fx = apply(
        &mut m,
        Msg::Recall(RecallMsg::OcclusionCard(
            String::new(),
            "assets/a.png".into(),
        )),
    );
    assert!(fx.is_empty());
    assert!(m.state.notice.is_some());
    let fx = apply(
        &mut m,
        Msg::Recall(RecallMsg::OcclusionCard(
            "bio".into(),
            "assets/a.png".into(),
        )),
    );
    let want = Effect::Library(LibraryEffect::Recall(RecallEffect::CreateOcclusion(
        "bio".into(),
        "assets/a.png".into(),
    )));
    assert_eq!(fx, [want]);
}

#[test]
fn a_picture_becomes_a_card_whose_page_opens_and_whose_masks_are_saved() {
    let dir = temp_dir("occlusion-flow");
    let bio = dir.path.join("bio");
    std::fs::create_dir_all(bio.clone()).unwrap();
    std::fs::create_dir_all(dir.path.join("assets")).unwrap();
    std::fs::write(dir.path.join("assets/heart.png"), b"png").unwrap();
    let stack = format!(
        "---\nid: {}\nkind: stack\ntitle: Bio\n---\n",
        format_ulid(Uid(1))
    );
    std::fs::write(bio.join("_stack.md"), stack).unwrap();
    let mut w = worker_new(None);
    let mut m = Model::default();
    m.state.app.tutorial_offered = true;
    run_to_idle(
        &mut m,
        &mut w,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    let make = RecallMsg::OcclusionCard("bio".into(), "assets/heart.png".into());
    send(&mut m, &mut w, Msg::Recall(make));
    assert_eq!(
        m.state.screen,
        Screen::Editor,
        "its page opens to draw the masks"
    );
    let e = m.state.editor.as_ref().unwrap();
    let mut o = e.form.occlusion.clone().unwrap();
    o.masks.push(Mask {
        x: 100,
        y: 200,
        w: 3000,
        h: 1000,
        label: "Aorta".into(),
    });
    send(
        &mut m,
        &mut w,
        Msg::Edit(EditAction::Occlusion(Some(o.clone()))),
    );
    send(&mut m, &mut w, Msg::SaveClicked);
    let path = dir.path.join(&m.state.editor.as_ref().unwrap().path);
    let text = std::fs::read_to_string(path).unwrap();
    let saved = occlusion_of(&studyfrog_md::parse::parse_doc(&text).unwrap()).unwrap();
    assert_eq!(saved, o, "{text}");
    assert_eq!(
        std::fs::read(dir.path.join("assets/heart.png")).unwrap(),
        b"png",
        "the picture is untouched"
    );
}

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

#[test]
fn a_picture_saved_from_a_pdf_offers_an_occlusion_card() {
    use egui_kittest::kittest::Queryable;
    let dir = temp_dir("occlusion-pdf");
    let bio = dir.path.join("bio");
    std::fs::create_dir_all(&bio).unwrap();
    let stack = format!(
        "---\nid: {}\nkind: stack\ntitle: Bio\n---\n",
        format_ulid(Uid(1))
    );
    std::fs::write(bio.join("_stack.md"), stack).unwrap();
    std::fs::write(bio.join("three.pdf"), pdfmake::three()).unwrap();
    let mut w = worker_new(None);
    let mut m = Model::default();
    m.state.app.tutorial_offered = true;
    run_to_idle(
        &mut m,
        &mut w,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    let open = studyfrog_ui::pdf_types::PdfMsg::Open("bio/three.pdf".into(), 0);
    send(&mut m, &mut w, Msg::Pdf(open));
    let view = m.state.pdf.as_mut().unwrap();
    view.side = studyfrog_ui::pdf_types::Side::Extract;
    view.extract.folder = "bio".into();
    view.extract.pictures = vec!["assets/figure-1.png".into()];
    let mut h = common::views::harness(m);
    h.run_steps(3);
    h.get_by_label("Occlusion card from assets/figure-1.png")
        .click();
    h.run_steps(2);
    let want = Effect::Library(LibraryEffect::Recall(RecallEffect::CreateOcclusion(
        "bio".into(),
        "assets/figure-1.png".into(),
    )));
    assert!(h.state().1.contains(&want), "{:?}", h.state().1);
}
