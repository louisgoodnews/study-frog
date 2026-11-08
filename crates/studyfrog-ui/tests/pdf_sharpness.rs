#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! BUG-0010: the page being read is painted from a picture with as many texels as the screen has
//! pixels, at every zoom and fit mode, with the thumbnail list open, after resizing and at
//! several display densities. Checked on what a frame really paints (the texture's size against
//! the painted rectangle), not on the update's bookkeeping.

mod common;
mod pdf_frame_common;

use eframe::egui;
use pdf_frame_common::{Painted, frame_of, open_big, page_sharpness, send, settle, view};
use studyfrog_ui::pdf_types::{PdfMsg, Side, Zoom};

const ZOOMS: [Zoom; 8] = [
    Zoom::FitWidth,
    Zoom::FitPage,
    Zoom::Percent(25),
    Zoom::Percent(50),
    Zoom::Percent(100),
    Zoom::Percent(150),
    Zoom::Percent(200),
    Zoom::Percent(400),
];

/// Within the 5 % that the scale's rounding (steps of 0.05 pixels per point) allows, unless the
/// page is drawn at the largest size a texture or the renderer allows (then it fills that limit).
fn assert_sharp(page: Painted, ratio: f32, max_side: usize, what: &str) {
    let [w, h] = page.texels;
    let pixels = (w * h) as f32 / studyfrog_pdf::limits::MAX_RENDER_PIXELS as f32;
    let at_limit = w.max(h) as f32 >= 0.95 * max_side as f32 || pixels >= 0.9;
    assert!(
        (0.95..=1.05).contains(&ratio) || (ratio < 0.95 && at_limit),
        "{what}: {ratio:.3} texels per screen pixel ({w}x{h})"
    );
    assert!(
        w.max(h) <= max_side,
        "{what}: {w}x{h} is over the texture limit"
    );
}

fn measure(f: &mut pdf_frame_common::Frames, what: &str) {
    let first = frame_of(f, Vec::new());
    settle(f, first);
    let (page, ratio) = page_sharpness(f);
    eprintln!(
        "{what}: {}x{} texels on {:.0}x{:.0} points, {ratio:.3}",
        page.texels[0],
        page.texels[1],
        page.rect.width(),
        page.rect.height()
    );
    assert_sharp(page, ratio, f.max_side, what);
}

#[test]
fn every_zoom_and_fit_mode_paints_the_page_at_screen_resolution_with_thumbnails_open() {
    let mut f = open_big("sharp-zoom");
    send(&mut f, PdfMsg::ShowSide(Side::Pages));
    for ppp in [1.0, 1.5, 2.0] {
        f.ctx.set_pixels_per_point(ppp);
        for zoom in ZOOMS {
            send(&mut f, PdfMsg::SetZoom(zoom));
            measure(&mut f, &format!("{zoom:?} at {ppp}x"));
        }
    }
    assert!(view(&f).pictures.iter().any(|(k, _)| k.scale_milli == 250));
}

#[test]
fn resizing_the_window_and_switching_fit_modes_brings_a_sharp_picture_again() {
    let mut f = open_big("sharp-resize");
    send(&mut f, PdfMsg::ShowSide(Side::Pages));
    for (zoom, size) in [
        (Zoom::FitWidth, egui::vec2(1200.0, 800.0)),
        (Zoom::FitWidth, egui::vec2(800.0, 600.0)),
        (Zoom::FitPage, egui::vec2(800.0, 600.0)),
        (Zoom::FitPage, egui::vec2(1600.0, 1000.0)),
        (Zoom::FitWidth, egui::vec2(1600.0, 1000.0)),
        (Zoom::FitPage, egui::vec2(1000.0, 1400.0)),
    ] {
        f.size = size;
        send(&mut f, PdfMsg::SetZoom(zoom));
        measure(&mut f, &format!("{zoom:?} in {}x{}", size.x, size.y));
    }
}

#[test]
fn going_to_another_page_with_thumbnails_cached_still_paints_it_sharp() {
    let mut f = open_big("sharp-pages");
    send(&mut f, PdfMsg::ShowSide(Side::Pages));
    send(&mut f, PdfMsg::SetZoom(Zoom::FitWidth));
    measure(&mut f, "page 1");
    for page in [1, 2, 3] {
        send(&mut f, PdfMsg::GoTo(page));
        measure(&mut f, &format!("page {}", page + 1));
    }
}

#[test]
fn on_a_card_with_small_textures_the_largest_zooms_are_capped_not_oversized() {
    let mut f = open_big("sharp-cap");
    f.max_side = 2048;
    f.ctx.set_pixels_per_point(2.0);
    send(&mut f, PdfMsg::ShowSide(Side::Pages));
    for zoom in [
        Zoom::Percent(150),
        Zoom::Percent(200),
        Zoom::Percent(400),
        Zoom::FitWidth,
    ] {
        send(&mut f, PdfMsg::SetZoom(zoom));
        measure(&mut f, &format!("{zoom:?} at 2x, 2048 texels at most"));
    }
    assert!(view(&f).error.is_none(), "{:?}", view(&f).error);
}
