#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The geometry of the image occlusion editor (STORY-0109).

use studyfrog_md::occlusion::{Mask, Occlusion, OcclusionMode};
use studyfrog_ui::occlusion_ui::{Gesture, finish, hit, preview, relabel, remove, start};

fn mask(x: u16, y: u16, w: u16, h: u16) -> Mask {
    Mask {
        x,
        y,
        w,
        h,
        label: "L".into(),
    }
}

fn two() -> Occlusion {
    Occlusion {
        image: "a.png".into(),
        mode: OcclusionMode::HideAll,
        masks: vec![mask(1000, 1000, 2000, 2000), mask(2000, 2000, 2000, 2000)],
    }
}

#[test]
fn the_topmost_mask_is_hit_and_a_press_starts_the_right_gesture() {
    let o = two();
    assert_eq!(
        hit(&o.masks, (0.25, 0.25)),
        Some(1),
        "the later mask lies on top"
    );
    assert_eq!(hit(&o.masks, (0.15, 0.15)), Some(0));
    assert_eq!(hit(&o.masks, (0.9, 0.9)), None);
    assert!(matches!(
        start(&o.masks, None, (0.9, 0.9)),
        Gesture::New(..)
    ));
    assert!(matches!(
        start(&o.masks, None, (0.15, 0.15)),
        Gesture::Move(0, ..)
    ));
    assert_eq!(
        start(&o.masks, Some(1), (0.4, 0.4)),
        Gesture::Resize(1),
        "the corner of the selected mask"
    );
}

#[test]
fn drawing_moving_and_resizing_change_the_masks_inside_the_picture() {
    let o = two();
    let (drawn, i) = finish(&o, Gesture::New(0.8, 0.8), (0.6, 0.7)).unwrap();
    assert_eq!(i, 2);
    assert_eq!(
        drawn.masks[2],
        Mask {
            x: 6000,
            y: 7000,
            w: 2000,
            h: 1000,
            label: String::new()
        }
    );
    assert!(
        finish(&o, Gesture::New(0.5, 0.5), (0.502, 0.6)).is_none(),
        "a tiny box is not kept"
    );
    let g = start(&o.masks, None, (0.15, 0.15));
    let (moved, _) = finish(&o, g, (2.0, 2.0)).unwrap();
    assert_eq!(
        (moved.masks[0].x, moved.masks[0].y),
        (8000, 8000),
        "kept inside the picture"
    );
    assert_eq!(moved.masks[0].label, "L");
    let (grown, _) = finish(&o, Gesture::Resize(1), (0.9, 0.5)).unwrap();
    assert_eq!((grown.masks[1].w, grown.masks[1].h), (7000, 3000));
    assert_eq!(preview(&o.masks, Gesture::Resize(9), (0.5, 0.5)), None);
}

#[test]
fn a_mask_is_labelled_and_deleted() {
    let o = two();
    assert_eq!(relabel(&o, 1, "Thalamus\nx").masks[1].label, "Thalamus x");
    assert_eq!(remove(&o, 0).masks, [mask(2000, 2000, 2000, 2000)]);
    assert_eq!(remove(&o, 7), o);
}
