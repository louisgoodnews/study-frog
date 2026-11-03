#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Every palette meets the contrast rules for text and controls.

use eframe::egui;
use studyfrog_ui::theme::{Look, palette_with};

fn luminance(c: egui::Color32) -> f64 {
    let f = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
}

fn contrast(a: egui::Color32, b: egui::Color32) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[test]
fn every_palette_meets_the_contrast_rules_for_text_and_controls() {
    for dark in [true, false] {
        for high_contrast in [false, true] {
            for accent in studyfrog_core::appearance::ACCENT_LIST {
                let p = palette_with(
                    dark,
                    Look {
                        accent,
                        high_contrast,
                    },
                );
                let who = format!("dark {dark}, high contrast {high_contrast}, {accent:?}");
                let text_goal = if high_contrast { 7.0 } else { 4.5 };
                for surface in [p.background, p.panel, p.surface] {
                    assert!(
                        contrast(p.text, surface) >= text_goal,
                        "text on {surface:?}: {who}"
                    );
                    assert!(
                        contrast(p.text_weak, surface) >= 4.5,
                        "weak text on {surface:?}: {who}"
                    );
                    assert!(
                        contrast(p.accent, surface) >= 3.0,
                        "accent on {surface:?}: {who}"
                    );
                }
                assert!(
                    contrast(p.on_accent, p.accent) >= 4.5,
                    "text on the accent: {who}"
                );
                assert!(
                    contrast(p.border, p.panel) >= if high_contrast { 4.5 } else { 1.2 },
                    "border: {who}"
                );
                for c in [p.success, p.warning, p.danger] {
                    assert!(contrast(c, p.surface) >= 3.0, "status colour {c:?}: {who}");
                }
            }
        }
    }
}
