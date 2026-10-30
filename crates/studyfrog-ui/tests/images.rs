#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Pictures in Markdown: parsing, which paths may be shown, and the helper text edits.

use studyfrog_md::blocks::{Block, Link, parse_blocks};
use studyfrog_ui::images::{append_line, image_markdown, image_uri};

#[test]
fn an_image_becomes_a_span_with_its_description_and_path() {
    let blocks = parse_blocks("Look: ![A map](assets/map.png) and ![](b.jpg)");
    let Block::Paragraph(spans) = &blocks[0] else {
        panic!("{blocks:?}")
    };
    let images: Vec<(&str, &Link)> = spans
        .iter()
        .filter_map(|s| s.link.as_ref().map(|l| (s.text.as_str(), l)))
        .collect();
    assert_eq!(
        images,
        [
            ("A map", &Link::Image("assets/map.png".into())),
            ("", &Link::Image("b.jpg".into()))
        ]
    );
}

#[test]
fn only_plain_relative_picture_paths_are_shown() {
    let root = "/home/me/vault/";
    assert_eq!(
        image_uri(root, "assets/a.PNG").as_deref(),
        Some("file:///home/me/vault/assets/a.PNG")
    );
    for bad in [
        "../x.png",
        "/etc/x.png",
        "a/../../x.png",
        "http://x/y.png",
        "C:\\x.png",
        "assets/a.svg",
        "assets/a",
        "",
        "./a.png",
    ] {
        assert_eq!(image_uri(root, bad), None, "{bad}");
    }
}

#[test]
fn the_inserted_markdown_names_the_picture_and_sits_on_its_own_line() {
    assert_eq!(
        image_markdown("assets/my-map-1a2b3c4d.png"),
        "![my-map-1a2b3c4d](assets/my-map-1a2b3c4d.png)"
    );
    assert_eq!(append_line("", "X"), "X\n");
    assert_eq!(append_line("text", "X"), "text\n\nX\n");
    assert_eq!(append_line("text\n", "X"), "text\n\nX\n");
    assert_eq!(append_line("text\n\n", "X"), "text\n\nX\n");
}

mod flow {
    use std::path::PathBuf;
    use studyfrog_core::uid::Uid;
    use studyfrog_service::create_more::NewKind;
    use studyfrog_service::document::LoadedDoc;
    use studyfrog_ui::effect::Effect;
    use studyfrog_ui::exec::{execute, worker_new};
    use studyfrog_ui::image_ui::ImageTarget;
    use studyfrog_ui::modal::{Field, Modal, ModalMsg};
    use studyfrog_ui::model::{Model, apply};
    use studyfrog_ui::msg::Msg;

    fn editing() -> Model {
        let mut m = Model::default();
        let doc = studyfrog_md::parse::parse_doc(
            "---\nid: 0000000000000000000000000A\nkind: note\ntitle: N\n---\nHello\n",
        )
        .unwrap();
        let loaded = LoadedDoc {
            path: "s/n.md".into(),
            doc,
            hash: 1,
        };
        apply(&mut m, Msg::VaultOpened(Ok(info())));
        apply(&mut m, Msg::DocumentLoaded(Uid(0xA), Ok(loaded)));
        m
    }

    fn info() -> studyfrog_ui::state::VaultInfo {
        let dir = std::env::temp_dir().join("studyfrog-ui-picture-info");
        studyfrog_ui::state::VaultInfo {
            root: dir.display().to_string(),
            name: "v".into(),
            overview: studyfrog_store::overview::VaultOverview::default(),
            settings: studyfrog_core::settings::vault_settings_default(),
            settings_error: None,
        }
    }

    #[test]
    fn a_finished_picture_goes_at_the_end_of_the_text_area_and_marks_the_page_changed() {
        let mut m = editing();
        let fx = apply(&mut m, Msg::PickImage(ImageTarget::Item(0)));
        assert_eq!(fx, [Effect::AddImage(ImageTarget::Item(0))]);
        apply(
            &mut m,
            Msg::ImageAdded(ImageTarget::Item(0), Ok(Some("assets/map-1.png".into()))),
        );
        let e = m.state.editor.as_ref().unwrap();
        assert_eq!(
            e.form.fields[0].text,
            "Hello\n\n![map-1](assets/map-1.png)\n"
        );
        assert!(studyfrog_ui::editor::is_dirty(e));
        apply(&mut m, Msg::ImageAdded(ImageTarget::Item(0), Ok(None)));
        assert_eq!(
            m.state.editor.as_ref().unwrap().form.fields[0]
                .text
                .matches("![")
                .count(),
            1,
            "cancel adds nothing"
        );
        apply(
            &mut m,
            Msg::ImageAdded(ImageTarget::Item(0), Err("too big".into())),
        );
        assert!(m.state.notice.as_ref().unwrap().text.contains("too big"));
    }

    #[test]
    fn the_create_dialog_gets_the_picture_in_the_right_field() {
        let mut m = editing();
        apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
        apply(&mut m, Msg::Modal(ModalMsg::SetKind(NewKind::Card)));
        apply(
            &mut m,
            Msg::ImageAdded(
                ImageTarget::Create(Field::Answer),
                Ok(Some("assets/a.png".into())),
            ),
        );
        let Some(Modal::Create(f)) = &m.state.modal else {
            panic!()
        };
        assert_eq!(
            (f.text.as_str(), f.answer.as_str()),
            ("", "![a](assets/a.png)\n")
        );
    }

    fn picker(_: &[&str]) -> Option<PathBuf> {
        Some(
            std::env::temp_dir()
                .join("studyfrog-ui-picture-source")
                .join("pic.png"),
        )
    }

    #[test]
    fn the_worker_copies_the_chosen_picture_into_the_vault() {
        let base = std::env::temp_dir();
        let (vault, source) = (
            base.join("studyfrog-ui-picture-vault"),
            base.join("studyfrog-ui-picture-source"),
        );
        std::fs::create_dir_all(&vault).unwrap();
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("pic.png"), b"png").unwrap();
        let mut w = worker_new(None);
        w.file_picker = Some(picker);
        execute(
            &mut w,
            Effect::OpenVault(vault.display().to_string()),
            1_800_000_000_000_000_000,
        );
        let msgs = execute(
            &mut w,
            Effect::AddImage(ImageTarget::Item(0)),
            1_800_000_000_000_000_000,
        );
        let Msg::ImageAdded(_, Ok(Some(rel))) = &msgs[0] else {
            panic!("{:?}", msgs[0])
        };
        assert!(rel.starts_with("assets/pic-") && vault.join(rel).is_file());
        let _ = std::fs::remove_dir_all(&vault);
        let _ = std::fs::remove_dir_all(&source);
    }
}
