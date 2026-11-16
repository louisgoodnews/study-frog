#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Automated tests for split-screen PDF navigation and highlight overlay (EPIC-051).

mod common;

use std::f32::consts::PI;

use studyfrog_pdf::crop::{PdfCropAnchor, parse_pdf_crop_uri};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_split_ui::{clamp_page_index, close_split, open_crop, set_split_ratio};
use studyfrog_ui::pdf_types::{PdfEffect, PdfMsg};
use studyfrog_ui::state::State;
use studyfrog_ui::view::pdf_highlight::{anchor_to_points, highlight_alpha};

#[test]
fn jump_button_rendering_condition_based_on_source_uris() {
    // Valid standard crop URI
    let uri = "textbooks/histology.pdf#page=84&bbox=150.0,220.0,500.0,480.0";
    let anchor = parse_pdf_crop_uri(uri).expect("valid crop URI should parse");
    assert_eq!(anchor.path, "textbooks/histology.pdf");
    assert_eq!(anchor.page, 84);
    assert_eq!(
        (anchor.x, anchor.y, anchor.w, anchor.h),
        (150.0, 220.0, 500.0, 480.0)
    );

    // Valid URI with "source:" prefix and quotes
    let uri_prefixed = "\"source: lectures/anatomy.pdf#page=12&bbox=10,20,30,40\"";
    let anchor2 = parse_pdf_crop_uri(uri_prefixed).expect("prefixed URI should parse");
    assert_eq!(anchor2.path, "lectures/anatomy.pdf");
    assert_eq!(anchor2.page, 12);
    assert_eq!(
        (anchor2.x, anchor2.y, anchor2.w, anchor2.h),
        (10.0, 20.0, 30.0, 40.0)
    );

    // Non-crop standard source links must not parse into crop anchors
    assert!(parse_pdf_crop_uri("textbooks/histology.pdf#page=84").is_none());
    assert!(parse_pdf_crop_uri("textbooks/histology.pdf").is_none());
    assert!(parse_pdf_crop_uri("").is_none());
    assert!(parse_pdf_crop_uri("notes/summary.md").is_none());

    // Malformed bboxes with negative dimensions or non-numbers
    assert!(parse_pdf_crop_uri("doc.pdf#page=1&bbox=0,0,-5,10").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#page=1&bbox=foo,bar,10,20").is_none());
}

#[test]
fn split_view_state_initialization_and_active_toggle() {
    let state = State {
        now: 42.0,
        ..Default::default()
    };

    let anchor = PdfCropAnchor {
        path: "textbooks/histology.pdf".into(),
        page: 15,
        x: 100.0,
        y: 200.0,
        w: 300.0,
        h: 250.0,
    };

    // Open crop initializes split view
    let (s2, effects) = open_crop(state, anchor.clone());
    assert!(s2.split_pdf.active);
    assert_eq!(s2.split_pdf.anchor, Some(anchor));
    assert_eq!(s2.split_pdf.split_ratio, 0.5);
    assert_eq!(s2.split_pdf.flash_start_now, 42.0);
    assert_eq!(
        effects,
        vec![Effect::Pdf(PdfEffect::Open(
            "textbooks/histology.pdf".into()
        ))]
    );
    assert_eq!(s2.pdf.as_ref().unwrap().page, 14); // 0-based page

    // Close split dismisses pane
    let (s3, effects_close) = close_split(s2.clone());
    assert!(!s3.split_pdf.active);
    assert!(effects_close.is_empty());

    // Adjusting split ratio clamps between 0.2 and 0.8
    let (s_ratio1, _) = set_split_ratio(s2.clone(), 0.65);
    assert_eq!(s_ratio1.split_pdf.split_ratio, 0.65);

    let (s_ratio_high, _) = set_split_ratio(s2.clone(), 0.95);
    assert_eq!(s_ratio_high.split_pdf.split_ratio, 0.8);

    let (s_ratio_low, _) = set_split_ratio(s2, 0.05);
    assert_eq!(s_ratio_low.split_pdf.split_ratio, 0.2);
}

#[test]
fn page_scroll_index_clamping() {
    let total = 10;
    assert_eq!(clamp_page_index(0, total), 0);
    assert_eq!(clamp_page_index(5, total), 5);
    assert_eq!(clamp_page_index(9, total), 9);
    assert_eq!(clamp_page_index(10, total), 9);
    assert_eq!(clamp_page_index(100, total), 9);

    // Empty document clamping
    assert_eq!(clamp_page_index(0, 0), 0);
    assert_eq!(clamp_page_index(5, 0), 0);
}

#[test]
fn pulsing_highlight_alpha_calculation_over_time() {
    let start = 100.0;

    // At t = 0 (elapsed = 0.0), sin(0) = 0 -> alpha = 0.3
    let a0 = highlight_alpha(start, 100.0);
    assert!((a0 - 0.3).abs() < 1e-4);

    // Quarter cycle of 6.0 rad/s: 6.0 * t = PI/2 => t = PI / 12 ~ 0.2618s
    let peak_t = start + (PI / 12.0) as f64;
    let a_peak = highlight_alpha(start, peak_t);
    assert!((a_peak - 0.9).abs() < 1e-3);

    // During first 3.0 seconds, alpha is strictly bounded [0.3, 0.9]
    for step in 0..30 {
        let now = start + (step as f64) * 0.1;
        let a = highlight_alpha(start, now);
        assert!(
            (0.3..=0.90001).contains(&a),
            "alpha {a} at step {step} out of bounds"
        );
    }

    // After 3.0 seconds, settles into steady 0.4
    assert_eq!(highlight_alpha(start, 103.0), 0.4);
    assert_eq!(highlight_alpha(start, 110.0), 0.4);

    // Negative elapsed time clamp
    assert!((highlight_alpha(start, 90.0) - 0.3).abs() < 1e-4);
}

#[test]
fn anchor_point_conversion_handles_normalized_and_points() {
    let page_bounds = (600.0, 800.0);

    // Point coordinates (> 1.0)
    let pt_anchor = PdfCropAnchor {
        path: "doc.pdf".into(),
        page: 1,
        x: 100.0,
        y: 150.0,
        w: 200.0,
        h: 250.0,
    };
    let pts = anchor_to_points(&pt_anchor, page_bounds);
    assert_eq!(pts, (100.0, 150.0, 200.0, 250.0));

    // Normalized coordinates (<= 1.0)
    let norm_anchor = PdfCropAnchor {
        path: "doc.pdf".into(),
        page: 1,
        x: 0.1,
        y: 0.2,
        w: 0.5,
        h: 0.4,
    };
    let converted = anchor_to_points(&norm_anchor, page_bounds);
    assert_eq!(converted, (60.0, 160.0, 300.0, 320.0));
}

#[test]
fn model_open_crop_and_close_split_messages() {
    let mut model = Model {
        state: State {
            now: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let anchor = PdfCropAnchor {
        path: "book.pdf".into(),
        page: 7,
        x: 50.0,
        y: 60.0,
        w: 120.0,
        h: 140.0,
    };

    let fx = apply(&mut model, Msg::Pdf(PdfMsg::OpenCrop(anchor.clone())));
    assert_eq!(fx, vec![Effect::Pdf(PdfEffect::Open("book.pdf".into()))]);
    assert!(model.state.split_pdf.active);
    assert_eq!(model.state.split_pdf.anchor, Some(anchor));

    apply(&mut model, Msg::Pdf(PdfMsg::CloseSplit));
    assert!(!model.state.split_pdf.active);
}
