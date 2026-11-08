//! The geometry of the image occlusion editor (STORY-0109): which mask a point hits, and what a drag does
//! to the masks. Points are fractions of the picture's width and height. Pure.

use studyfrog_md::occlusion::{Mask, Occlusion, mask_from_unit, unit_of};

/// What a drag on the picture does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// Draw a new mask from this corner.
    New(f32, f32),
    /// Move mask `0`, grabbed this far from its top-left corner.
    Move(usize, f32, f32),
    /// Resize mask `0` by its bottom-right corner.
    Resize(usize),
}

/// How close (as a fraction) to a mask's bottom-right corner a press resizes it.
pub const HANDLE: f32 = 0.025;
/// Smallest new mask, as a fraction of each side.
pub const MIN_SIDE: f32 = 0.01;

/// The topmost mask under `(x, y)` (the last one drawn wins).
pub fn hit(masks: &[Mask], (x, y): (f32, f32)) -> Option<usize> {
    masks.iter().rposition(|m| {
        let (mx, my, mw, mh) = unit_of(m);
        x >= mx && x <= mx + mw && y >= my && y <= my + mh
    })
}

/// The gesture a press at `p` starts, given the selected mask.
pub fn start(masks: &[Mask], selected: Option<usize>, p: (f32, f32)) -> Gesture {
    if let Some(i) = selected.filter(|i| *i < masks.len()) {
        let (mx, my, mw, mh) = unit_of(&masks[i]);
        if (p.0 - (mx + mw)).abs() <= HANDLE && (p.1 - (my + mh)).abs() <= HANDLE {
            return Gesture::Resize(i);
        }
    }
    match hit(masks, p) {
        Some(i) => {
            let (mx, my, _, _) = unit_of(&masks[i]);
            Gesture::Move(i, p.0 - mx, p.1 - my)
        }
        None => Gesture::New(p.0, p.1),
    }
}

/// The rectangle `(x, y, w, h)` a gesture shows with the pointer at `p`.
pub fn preview(masks: &[Mask], g: Gesture, p: (f32, f32)) -> Option<(f32, f32, f32, f32)> {
    let (px, py) = (p.0.clamp(0.0, 1.0), p.1.clamp(0.0, 1.0));
    match g {
        Gesture::New(x, y) => Some((x.min(px), y.min(py), (px - x).abs(), (py - y).abs())),
        Gesture::Move(i, gx, gy) => {
            let (_, _, w, h) = unit_of(masks.get(i)?);
            Some((
                (px - gx).clamp(0.0, 1.0 - w),
                (py - gy).clamp(0.0, 1.0 - h),
                w,
                h,
            ))
        }
        Gesture::Resize(i) => {
            let (x, y, _, _) = unit_of(masks.get(i)?);
            Some((x, y, (px - x).max(MIN_SIDE), (py - y).max(MIN_SIDE)))
        }
    }
}

/// The occlusion after the gesture ended at `p`, and the mask to select; `None` when nothing changes
/// (a new mask too small to keep).
pub fn finish(o: &Occlusion, g: Gesture, p: (f32, f32)) -> Option<(Occlusion, usize)> {
    let rect = preview(&o.masks, g, p)?;
    let mut out = o.clone();
    match g {
        Gesture::New(..) if rect.2 < MIN_SIDE || rect.3 < MIN_SIDE => None,
        Gesture::New(..) => {
            out.masks.push(mask_from_unit(rect, ""));
            let i = out.masks.len() - 1;
            Some((out, i))
        }
        Gesture::Move(i, ..) | Gesture::Resize(i) => {
            let label = out.masks[i].label.clone();
            out.masks[i] = mask_from_unit(rect, &label);
            (out != *o).then_some((out, i))
        }
    }
}

/// The occlusion with mask `i` labelled `label`.
pub fn relabel(o: &Occlusion, i: usize, label: &str) -> Occlusion {
    let mut out = o.clone();
    if let Some(m) = out.masks.get_mut(i) {
        m.label = label.replace('\n', " ");
    }
    out
}

/// The occlusion without mask `i`.
pub fn remove(o: &Occlusion, i: usize) -> Occlusion {
    let mut out = o.clone();
    if i < out.masks.len() {
        out.masks.remove(i);
    }
    out
}
