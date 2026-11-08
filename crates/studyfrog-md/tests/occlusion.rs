#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Image occlusion masks in a card's front matter: reading, writing and the lossless round trip (STORY-0109).

use proptest::prelude::*;
use studyfrog_md::occlusion::{
    Mask, Occlusion, OcclusionMode, UNIT, images_in, mask_from_unit, occlusion_of, parse_occlusion,
    render_occlusion, unit_of, with_occlusion, without_images,
};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;

const CARD: &str = "---\nid: 0000000000000000000000000A\nkind: card\ntitle: Brain\nocclusion:\n  image: \"assets/brain.png\"\n  mode: hide_one\n  masks:\n    - \"0.1200 0.3400 0.2000 0.0500 Hippocampus\"\n    - \"0.5000 0.5000 0.1000 0.1000\"\nx_other: kept\n---\n![brain](assets/brain.png)\n\nName the part.\n???\n\n";

#[test]
fn a_card_block_is_read_with_its_picture_mode_and_masks() {
    let doc = parse_doc(CARD).unwrap();
    let o = occlusion_of(&doc).unwrap();
    assert_eq!(o.image, "assets/brain.png");
    assert_eq!(o.mode, OcclusionMode::HideOne);
    assert_eq!(
        o.masks[0],
        Mask {
            x: 1200,
            y: 3400,
            w: 2000,
            h: 500,
            label: "Hippocampus".into()
        }
    );
    assert_eq!(o.masks[1].label, "");
    assert_eq!(
        render_doc(&doc).unwrap(),
        CARD,
        "the file is written back unchanged"
    );
}

#[test]
fn a_changed_block_replaces_the_old_one_and_keeps_other_keys() {
    let doc = parse_doc(CARD).unwrap();
    let mut o = occlusion_of(&doc).unwrap();
    o.mode = OcclusionMode::HideAll;
    o.masks.pop();
    let text = render_doc(&with_occlusion(&doc, Some(&o))).unwrap();
    assert!(
        text.contains("mode: hide_all") && text.contains("x_other: kept"),
        "{text}"
    );
    assert_eq!(occlusion_of(&parse_doc(&text).unwrap()), Some(o));
    let gone = render_doc(&with_occlusion(&doc, None)).unwrap();
    assert!(!gone.contains("occlusion:"));
}

#[test]
fn malformed_blocks_and_masks_outside_the_picture_are_not_read() {
    for bad in [
        "occlusion:\n  mode: hide_all\n",
        "occlusion:\n  image: a.png\n  mode: sideways\n",
        "occlusion:\n  image: a.png\n  masks:\n    - \"0.9 0.1 0.2 0.1\"\n",
        "occlusion:\n  image: a.png\n  masks:\n    - \"0.1 0.1 0 0.1\"\n",
        "occlusion:\n  image: a.png\n  masks:\n    - \"x 0.1 0.1 0.1\"\n",
    ] {
        assert!(parse_occlusion(bad).is_none(), "{bad}");
    }
}

#[test]
fn fractions_are_clamped_into_the_picture() {
    let m = mask_from_unit((0.95, -0.2, 0.3, 2.0), " Thalamus ");
    assert_eq!((m.x, m.y, m.w, m.h), (9500, 0, 500, UNIT));
    assert_eq!(m.label, "Thalamus");
    assert_eq!(unit_of(&m), (0.95, 0.0, 0.05, 1.0));
}

#[test]
fn pictures_are_found_and_taken_out_of_the_prompt() {
    let t = "![a](x/a.png) Name it ![b]( b.jpg )";
    assert_eq!(images_in(t), ["x/a.png", "b.jpg"]);
    assert_eq!(without_images(t), "Name it");
}

fn mask() -> impl Strategy<Value = Mask> {
    (0..UNIT, 0..UNIT, "[A-Za-z0-9 \"\\\\]{0,12}")
        .prop_flat_map(|(x, y, label)| {
            (
                Just(x),
                Just(y),
                1..=UNIT - x,
                1..=UNIT - y,
                Just(label.trim().to_owned()),
            )
        })
        .prop_map(|(x, y, w, h, label)| Mask { x, y, w, h, label })
}

proptest! {
    #[test]
    fn any_block_survives_writing_and_reading(
        masks in prop::collection::vec(mask(), 0..6),
        image in "[a-z]{1,8}/[a-z \"]{1,8}\\.png",
        one in any::<bool>(),
    ) {
        let o = Occlusion { image, mode: if one { OcclusionMode::HideOne } else { OcclusionMode::HideAll }, masks };
        prop_assert_eq!(parse_occlusion(&render_occlusion(&o)), Some(o.clone()));
        let doc = parse_doc("---\nid: 0000000000000000000000000A\nkind: card\ntitle: T\n---\nq\n???\na\n").unwrap();
        let with = with_occlusion(&doc, Some(&o));
        let back = parse_doc(&render_doc(&with).unwrap()).unwrap();
        prop_assert_eq!(&back, &with, "lossless round trip");
        prop_assert_eq!(occlusion_of(&back), Some(o));
    }
}
