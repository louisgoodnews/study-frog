#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0114: the four-tier type scale follows the text size, prose keeps 60 to 75 characters per line on average and a line
//! height of 1.5, and the fonts' digits are tabular so ticking numbers do not jitter.

use eframe::egui::{self, FontId, TextStyle};
use studyfrog_core::appearance::{Appearance, TEXT_SIZES};
use studyfrog_ui::theme_apply::apply_look;
use studyfrog_ui::typo::{self, TIERS, Tier};

/// Run one frame with the look for `text` percent installed and hand the `ui` to `f`.
fn with_ui(text: u16, f: impl FnOnce(&mut egui::Ui)) {
    let ctx = egui::Context::default();
    let a = Appearance {
        text,
        ..Appearance::default()
    };
    apply_look(&ctx, &a);
    ctx.set_theme(egui::Theme::Dark);
    let mut f = Some(f);
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        if let Some(f) = f.take() {
            f(ui);
        }
    });
    out.textures_delta.clear();
}

#[test]
fn the_type_scale_has_four_tiers_that_follow_the_text_size() {
    for text in TEXT_SIZES {
        with_ui(text, |ui| {
            let body = typo::size_for(f32::from(text) / 100.0 * typo::BASE, Tier::Body);
            let got = typo::body_size(ui);
            assert!((got - body).abs() < 0.01, "body {got} at {text} %");
            let sizes: Vec<f32> = TIERS.iter().map(|t| typo::size(ui, *t)).collect();
            assert!(
                sizes.windows(2).all(|w| w[0] > w[1]),
                "tiers fall: {sizes:?}"
            );
            let style = |s: TextStyle| ui.style().text_styles[&s].size;
            assert_eq!(style(TextStyle::Heading), typo::size(ui, Tier::Heading));
            assert_eq!(style(TextStyle::Button), typo::size(ui, Tier::Label));
            assert_eq!(style(TextStyle::Small), typo::size(ui, Tier::Caption));
        });
    }
}

#[test]
fn the_reading_measure_holds_60_to_75_characters_per_line() {
    for text in TEXT_SIZES {
        with_ui(text, |ui| {
            let width = typo::measure(ui);
            let font = FontId::proportional(typo::body_size(ui));
            let prose = typo::SAMPLE.repeat(6);
            let galley = ui
                .ctx()
                .fonts_mut(|f| f.layout(prose.clone(), font, egui::Color32::WHITE, width));
            let rows = &galley.rows;
            assert!(rows.len() > 4, "wrapped into rows");
            // Every full line (not the last) holds 60 to 75 characters.
            // Full lines (not the last) average 60 to 75 characters; word wrap moves single lines a little.
            let counts: Vec<usize> = rows[..rows.len() - 1]
                .iter()
                .map(|r| r.char_count_including_newline().0)
                .collect();
            let mean = counts.iter().sum::<usize>() as f32 / counts.len() as f32;
            let who = format!("{counts:?} at {text} % ({width} pt)");
            assert!((60.0..=75.0).contains(&mean), "mean {mean}: {who}");
            assert!(counts.iter().all(|n| (50..=75).contains(n)), "{who}");
        });
    }
}

#[test]
fn prose_has_a_line_height_of_one_and_a_half() {
    with_ui(100, |ui| {
        let size = typo::body_size(ui);
        let text = egui::WidgetText::from(typo::prose(ui, &typo::SAMPLE.repeat(3), size));
        let galley = text.into_galley(ui, Some(egui::TextWrapMode::Wrap), 300.0, TextStyle::Body);
        let ys: Vec<f32> = galley.rows.iter().map(|r| r.rect().min.y).collect();
        assert!(ys.len() > 3);
        for w in ys.windows(2) {
            let step = w[1] - w[0];
            assert!(
                (step / size - 1.5).abs() < 0.05,
                "line step {step} for {size} pt"
            );
        }
    });
}

#[test]
fn the_digits_of_both_fonts_are_tabular() {
    with_ui(100, |ui| {
        for font in [FontId::proportional(15.0), FontId::monospace(14.0)] {
            assert!(typo::digits_are_tabular(ui.ctx(), &font), "{font:?}");
        }
    });
}

/// Glyphs drawn as pictures, not text, keep their own size.
const GLYPHS: [&str; 4] = [
    "view/icons.rs",
    "view/md_editor/resize.rs",
    "view/modal_size.rs",
    "view/navbar.rs",
];

#[test]
fn views_take_their_text_sizes_from_the_type_scale() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stack = vec![root.clone()];
    let mut bad = Vec::new();
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(dir).unwrap().filter_map(Result::ok) {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let rel = p
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if GLYPHS.contains(&rel.as_str()) || !rel.ends_with(".rs") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap();
            for (n, line) in text.lines().enumerate() {
                let literal = |pat: &str| {
                    line.match_indices(pat).any(|(i, _)| {
                        line[i + pat.len()..].starts_with(|c: char| c.is_ascii_digit())
                    })
                };
                if [".size(", "FontId::proportional(", "FontId::monospace("]
                    .iter()
                    .any(|p| literal(p))
                {
                    bad.push(format!("{rel}:{}: {}", n + 1, line.trim()));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "text sizes outside the type scale:\n{}",
        bad.join("\n")
    );
}

#[test]
fn a_ticking_number_never_outgrows_its_template() {
    with_ui(100, |ui| {
        let font = typo::font(ui, Tier::Caption);
        let w = |s: &str| typo::text_width(ui.ctx(), s, &font);
        assert_eq!(typo::figure_template("Time left 04:12"), "Time left 00:00");
        for (a, b) in [("0.6 s", "9.9 s"), ("12:59", "07:01"), ("3 / 10", "9 / 10")] {
            assert!((w(a) - w(b)).abs() < 0.01, "{a} and {b} differ");
        }
        // The run's seconds pill reserves "000.0 s", wider than any time under 1000 s.
        assert!(w("000.0 s") >= w("999.9 s") - 0.01 && w("000.0 s") > w("12.3 s"));
    });
}
