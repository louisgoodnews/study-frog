#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)] // tests may panic; they wait for the loader thread
//! The image occlusion editor on a card's page and the masked picture in a run, with real pictures (STORY-0109).

mod common;

use common::runs::act;
use common::views::{harness, info, open_model, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_md::occlusion::{Mask, OcclusionMode};
use studyfrog_rehearsal::input::Action;
use studyfrog_service::content::ItemContent;
use studyfrog_service::content_occlusion::OcclusionView;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn root() -> String {
    let dir = std::env::temp_dir().join(format!("studyfrog-occ-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    image::RgbaImage::from_pixel(400, 300, image::Rgba([200, 220, 240, 255]))
        .save(dir.join("assets/brain.png"))
        .unwrap();
    dir.display().to_string()
}

fn with_vault(mut m: Model, root: &str) -> Model {
    let mut v = info();
    v.root = root.into();
    m.state.vault = Some(v);
    m
}

fn settle(h: &mut egui_kittest::Harness<'_, common::views::App>) {
    egui_extras::install_image_loaders(&h.ctx);
    for _ in 0..8 {
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}

fn card_page(root: &str) -> Model {
    let text = "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Brain\n---\n![brain](assets/brain.png)\n???\n\n";
    let doc = studyfrog_md::parse::parse_doc(text).unwrap();
    let mut m = with_vault(open_model(), root);
    let loaded = LoadedDoc {
        path: "s/brain.md".into(),
        doc,
        hash: 1,
    };
    apply(&mut m, Msg::DocumentLoaded(Uid(0xA), Ok(loaded)));
    m
}

#[test]
fn a_card_with_a_picture_becomes_an_occlusion_card_and_a_drag_draws_a_mask() {
    let root = root();
    let mut h = egui_kittest::Harness::builder()
        .with_size(eframe::egui::vec2(1000.0, 1600.0))
        .build_ui_state(
            |ui, (model, effects): &mut common::views::App| {
                effects.extend(studyfrog_ui::model::frame(model, ui))
            },
            (card_page(&root), Vec::new()),
        );
    settle(&mut h);
    h.get_by_label("Make this an image occlusion card").click();
    settle(&mut h);
    let o = h
        .state()
        .0
        .state
        .editor
        .as_ref()
        .unwrap()
        .form
        .occlusion
        .clone()
        .unwrap();
    assert_eq!((o.image.as_str(), o.masks.len()), ("assets/brain.png", 0));
    let pic = h
        .get_by_label("Picture with 0 masks: drag to draw a mask")
        .rect();
    assert!(pic.max.y < 1600.0, "{pic:?}");
    let (from, to) = (pic.min + (pic.size() * 0.2), pic.min + (pic.size() * 0.5));
    h.hover_at(from);
    h.run();
    h.drag_at(from);
    h.run();
    for step in 1..=6 {
        h.hover_at(from + (to - from) * (step as f32 / 6.0));
        h.run();
    }
    h.drop_at(to);
    h.run_steps(3);
    let o = h
        .state()
        .0
        .state
        .editor
        .as_ref()
        .unwrap()
        .form
        .occlusion
        .clone()
        .unwrap();
    assert_eq!(o.masks.len(), 1, "one mask drawn");
    let m = &o.masks[0];
    let near = |v: u16, want: u16| v.abs_diff(want) <= 300;
    assert!(
        near(m.x, 2000) && near(m.y, 2000) && near(m.w, 3000) && near(m.h, 3000),
        "{m:?}"
    );
    h.get_by_label("Label of mask 1").focus();
    h.get_by_label("Label of mask 1").type_text("Hippocampus");
    h.run_steps(2);
    let o = h
        .state()
        .0
        .state
        .editor
        .as_ref()
        .unwrap()
        .form
        .occlusion
        .clone()
        .unwrap();
    assert_eq!(o.masks[0].label, "Hippocampus");
    h.get_by_label("Hide one, guess one").click();
    h.run_steps(2);
    let e = h.state().0.state.editor.as_ref().unwrap();
    assert_eq!(
        e.form.occlusion.as_ref().unwrap().mode,
        OcclusionMode::HideOne
    );
    let doc = studyfrog_ui::edit_form::apply_form(&e.doc, &e.form).unwrap();
    let text = studyfrog_md::render::render_doc(&doc).unwrap();
    assert!(
        text.contains("mode: hide_one") && text.contains("Hippocampus\""),
        "{text}"
    );
    h.get_by_label("Delete mask 1").click();
    h.run_steps(2);
    let o = h
        .state()
        .0
        .state
        .editor
        .as_ref()
        .unwrap()
        .form
        .occlusion
        .clone()
        .unwrap();
    assert!(o.masks.is_empty(), "deleted");
}

fn occluded_run(root: &str, mode: RunMode) -> Model {
    let mut m = with_vault(running_model(mode), root);
    let masks = vec![
        Mask {
            x: 1000,
            y: 1000,
            w: 2000,
            h: 1000,
            label: "Hippocampus".into(),
        },
        Mask {
            x: 5000,
            y: 5000,
            w: 2000,
            h: 1000,
            label: "Thalamus".into(),
        },
    ];
    let content = ItemContent {
        title: "Brain".into(),
        prompt: "Name the part.".into(),
        answer: "**Hippocampus**".into(),
        previous: None,
        source: None,
        color: None,
        expected: Some("Hippocampus".into()),
        occlusion: Some(Box::new(OcclusionView {
            image: "assets/brain.png".into(),
            mode: OcclusionMode::HideAll,
            masks,
            active: 0,
        })),
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content)));
    m
}

#[test]
fn a_run_hides_and_marks_the_asked_mask_and_uncovers_it_on_the_flip() {
    let root = root();
    let mut h = harness(occluded_run(&root, RunMode::Flip));
    settle(&mut h);
    assert!(
        h.query_by_label("Picture with 2 masks, mask 1 hidden and marked")
            .is_some()
    );
    assert!(h.query_by_label("Hippocampus").is_none());
    let mut m = std::mem::take(&mut h.state_mut().0);
    act(&mut m, 1.0, Action::Flip);
    let mut h = harness(m);
    settle(&mut h);
    assert!(
        h.query_by_label("Picture with 2 masks, mask 1 uncovered")
            .is_some()
    );
    assert!(
        h.query_by_label("Hippocampus").is_some(),
        "the label is the answer"
    );
}

#[test]
fn a_typed_answer_is_compared_with_the_label() {
    let root = root();
    let mut m = occluded_run(&root, RunMode::Answer);
    let fx = act(&mut m, 1.0, Action::SubmitAnswer("hippocampus".into()));
    assert!(fx.iter().any(|e| matches!(e,
        studyfrog_ui::effect::Effect::ScoreAnswer { answer, .. }
            if answer.1.as_deref() == Some("Hippocampus"))));
}
