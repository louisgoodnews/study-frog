#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for PDF rectangular crop selection and frontmatter schema.

use studyfrog_pdf::crop::{
    PdfCropAnchor, compute_crop_rect, crop_rect_area, format_pdf_crop_uri, is_valid_crop_anchor,
    is_valid_crop_rect, normalized_to_points, parse_pdf_crop_uri, points_to_normalized,
};

#[test]
fn format_and_parse_exact_prompt_uri() {
    let raw = "textbooks/histology.pdf#page=84&bbox=150.0,220.0,500.0,480.0";
    let anchor = parse_pdf_crop_uri(raw).expect("should parse valid URI");
    assert_eq!(
        anchor,
        PdfCropAnchor {
            path: "textbooks/histology.pdf".to_string(),
            page: 84,
            x: 150.0,
            y: 220.0,
            w: 500.0,
            h: 480.0,
        }
    );
    let formatted = format_pdf_crop_uri(&anchor);
    assert_eq!(formatted, raw);
}

#[test]
fn roundtrip_format_and_parse() {
    let anchor = PdfCropAnchor {
        path: "notes/diagram.pdf".to_string(),
        page: 3,
        x: 12.5,
        y: 45.25,
        w: 100.0,
        h: 200.75,
    };
    let formatted = format_pdf_crop_uri(&anchor);
    let parsed = parse_pdf_crop_uri(&formatted).expect("should parse formatted uri");
    assert_eq!(anchor, parsed);
}

#[test]
fn parse_frontmatter_prefix_and_quotes() {
    let uri1 = "source: textbooks/histology.pdf#page=84&bbox=150.0,220.0,500.0,480.0";
    let parsed1 = parse_pdf_crop_uri(uri1).expect("should strip source: prefix");
    assert_eq!(parsed1.page, 84);

    let uri2 = "\"textbooks/histology.pdf#page=84&bbox=150.0,220.0,500.0,480.0\"";
    let parsed2 = parse_pdf_crop_uri(uri2).expect("should strip quotes");
    assert_eq!(parsed2.path, "textbooks/histology.pdf");

    let uri3 = "  source: 'docs/test.pdf#page=1&bbox=0.0,0.0,10.0,20.0' ";
    let parsed3 = parse_pdf_crop_uri(uri3).expect("should strip quotes and source:");
    assert_eq!(parsed3.path, "docs/test.pdf");
    assert_eq!(parsed3.page, 1);
}

#[test]
fn parse_param_order_independence_and_integers() {
    let uri = "paper.pdf#bbox=150,220,500,480&page=42";
    let parsed = parse_pdf_crop_uri(uri).expect("should parse reversed order and integer coords");
    assert_eq!(parsed.page, 42);
    assert_eq!(parsed.x, 150.0);
    assert_eq!(parsed.y, 220.0);
    assert_eq!(parsed.w, 500.0);
    assert_eq!(parsed.h, 480.0);
}

#[test]
fn parse_invalid_uris() {
    assert!(parse_pdf_crop_uri("").is_none());
    assert!(parse_pdf_crop_uri("no_fragment.pdf").is_none());
    assert!(parse_pdf_crop_uri("#page=1&bbox=0,0,10,10").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#page=abc&bbox=0,0,10,10").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#bbox=0,0,10,10").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#page=1").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#page=1&bbox=0,0,10").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#page=1&bbox=0,0,10,10,10").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#page=1&bbox=0,0,-10,10").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#page=1&bbox=0,0,10,-10").is_none());
    assert!(parse_pdf_crop_uri("doc.pdf#page=1&bbox=NaN,0,10,10").is_none());
}

#[test]
fn compute_crop_rect_drag_directions() {
    let page = (612.0, 792.0);

    // Top-left to bottom-right
    let rect1 = compute_crop_rect((100.0, 150.0), (300.0, 450.0), page);
    assert_eq!(rect1, (100.0, 150.0, 200.0, 300.0));

    // Bottom-right to top-left
    let rect2 = compute_crop_rect((300.0, 450.0), (100.0, 150.0), page);
    assert_eq!(rect2, (100.0, 150.0, 200.0, 300.0));

    // Top-right to bottom-left
    let rect3 = compute_crop_rect((300.0, 150.0), (100.0, 450.0), page);
    assert_eq!(rect3, (100.0, 150.0, 200.0, 300.0));

    // Bottom-left to top-right
    let rect4 = compute_crop_rect((100.0, 450.0), (300.0, 150.0), page);
    assert_eq!(rect4, (100.0, 150.0, 200.0, 300.0));
}

#[test]
fn compute_crop_rect_clamping_and_zero_area() {
    let page = (500.0, 500.0);

    // Start or current outside bounds
    let clamped = compute_crop_rect((-50.0, -100.0), (600.0, 700.0), page);
    assert_eq!(clamped, (0.0, 0.0, 500.0, 500.0));

    // Click without drag
    let zero = compute_crop_rect((100.0, 200.0), (100.0, 200.0), page);
    assert_eq!(zero, (100.0, 200.0, 0.0, 0.0));

    // Degenerate page bounds
    let degen = compute_crop_rect((10.0, 10.0), (20.0, 20.0), (-100.0, 0.0));
    assert_eq!(degen, (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn validation_helpers_and_area() {
    assert!(is_valid_crop_rect(0.0, 0.0, 100.0, 50.0));
    assert!(!is_valid_crop_rect(-1.0, 0.0, 100.0, 50.0));
    assert!(!is_valid_crop_rect(0.0, 0.0, 0.0, 50.0));
    assert!(!is_valid_crop_rect(0.0, 0.0, 100.0, -5.0));
    assert!(!is_valid_crop_rect(f32::NAN, 0.0, 100.0, 50.0));

    let anchor = PdfCropAnchor {
        path: "book.pdf".to_string(),
        page: 1,
        x: 10.0,
        y: 20.0,
        w: 30.0,
        h: 40.0,
    };
    assert!(is_valid_crop_anchor(&anchor));
    assert_eq!(crop_rect_area(anchor.w, anchor.h), 1200.0);

    let invalid_page = PdfCropAnchor {
        page: 0,
        ..anchor.clone()
    };
    assert!(!is_valid_crop_anchor(&invalid_page));

    let invalid_path = PdfCropAnchor {
        path: "   ".to_string(),
        ..anchor
    };
    assert!(!is_valid_crop_anchor(&invalid_path));
}

#[test]
fn normalized_points_conversions() {
    let page = (1000.0, 800.0);
    let pts = (100.0, 200.0, 500.0, 400.0);
    let norm = points_to_normalized(pts, page);
    assert_eq!(norm, (0.1, 0.25, 0.5, 0.5));
    let roundtrip = normalized_to_points(norm, page);
    assert_eq!(roundtrip, pts);
}
