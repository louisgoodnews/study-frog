#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)] // tests may panic; they wait for the loader thread
//! A picture in a note really loads (with the app's image loaders installed) and one outside the
//! vault is only named.

mod common;

use common::views::{harness, info, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn note_with(root: &str, text: &str) -> Model {
    let mut m = open_model();
    let mut v = info();
    v.root = root.into();
    m.state.vault = Some(v);
    let doc = studyfrog_md::parse::parse_doc(&format!(
        "---\nid: 0000000000000000000000000A\nkind: note\ntitle: N\n---\n{text}\n"
    ))
    .unwrap();
    apply(
        &mut m,
        Msg::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "s/n.md".into(),
                doc,
                hash: 1,
            }),
        ),
    );
    m
}

#[test]
fn a_picture_inside_the_vault_loads_and_one_outside_is_only_named() {
    let dir = std::env::temp_dir().join(format!("studyfrog-render-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    image::RgbaImage::from_pixel(4, 3, image::Rgba([10, 200, 30, 255]))
        .save(dir.join("assets/t.png"))
        .unwrap();
    let root = dir.display().to_string();
    let mut h = harness(note_with(
        &root,
        "![green](assets/t.png)\n\n![secret](../../etc/x.png)",
    ));
    egui_extras::install_image_loaders(&h.ctx);
    for _ in 0..6 {
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    assert!(
        h.query_all_by_label_contains("[picture: secret]").count() >= 1,
        "outside the vault: named, not loaded"
    );
    let uri = format!("file://{root}/assets/t.png");
    let loaded = h.ctx.try_load_texture(
        &uri,
        eframe::egui::TextureOptions::default(),
        eframe::egui::SizeHint::default(),
    );
    assert!(
        matches!(loaded, Ok(eframe::egui::load::TexturePoll::Ready { .. })),
        "the picture did not load: {:?}",
        loaded.err()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_or_unreadable_picture_is_named_and_never_crashes_the_window() {
    let dir = std::env::temp_dir().join(format!("studyfrog-render-missing-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("assets/bad.png"), b"not a png").unwrap();
    let root = dir.display().to_string();
    let mut h = harness(note_with(
        &root,
        "![gone](assets/gone.png)\n\n![bad](assets/bad.png)",
    ));
    h.run_steps(2); // no loaders installed yet
    egui_extras::install_image_loaders(&h.ctx);
    for _ in 0..6 {
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    assert!(h.query_all_by_label_contains("could not be shown").count() >= 2);
    let _ = std::fs::remove_dir_all(&dir);
}
