//! The coloured layout of the ScriptStudio text (STORY-0150): each highlighted piece in its colour, the error's line
//! underlined, and the bracket at the cursor and its match tinted.

use std::sync::Arc;

use eframe::egui::{self, Color32, FontId, Galley, Stroke, text::LayoutJob};
use studyfrog_script::editor::{Part, highlight, line_range, matching_bracket};

use crate::theme::{Palette, tint};

pub(super) fn colour(part: Part, p: &Palette) -> Color32 {
    match part {
        Part::Keyword => p.accent,
        Part::Builtin => p.success,
        Part::Text | Part::Number => p.warning,
        Part::Comment => p.text_weak,
        Part::Operator | Part::Plain => p.text,
    }
}

/// The coloured text: each piece in its colour, the error's line underlined, the bracket at the cursor and its match
/// tinted.
#[allow(clippy::too_many_arguments)]
pub(super) fn layout(
    ui: &egui::Ui,
    text: &str,
    cursor: Option<usize>,
    error_line: Option<usize>,
    wrap: f32,
    p: &Palette,
    font: &FontId,
) -> Arc<Galley> {
    let mut job = LayoutJob {
        text: text.to_owned(),
        ..Default::default()
    };
    job.wrap.max_width = wrap;
    // An error at the end of the text is underlined on the last line.
    let err = error_line
        .and_then(|l| line_range(text, l).or_else(|| line_range(text, text.split('\n').count())));
    let marks: Vec<usize> = cursor
        .and_then(|c| matching_bracket(text, c).map(|m| (c, m)))
        .map(|(c, m)| {
            let here = [c, c.saturating_sub(1)].into_iter().find(|&i| {
                text.as_bytes()
                    .get(i)
                    .is_some_and(|b| b"()[]{}".contains(b))
            });
            here.into_iter().chain(std::iter::once(m)).collect()
        })
        .unwrap_or_default();
    let mut cuts: Vec<usize> = highlight(text)
        .into_iter()
        .flat_map(|(r, _)| [r.start, r.end])
        .chain(err.iter().flat_map(|r| [r.start, r.end]))
        .chain(marks.iter().flat_map(|&m| [m, m + 1]))
        .chain([0, text.len()])
        .filter(|&i| i <= text.len() && text.is_char_boundary(i))
        .collect();
    cuts.sort_unstable();
    cuts.dedup();
    let parts = highlight(text);
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        if a == b {
            continue;
        }
        let part = parts
            .iter()
            .find(|(r, _)| r.contains(&a))
            .map_or(Part::Plain, |(_, p)| *p);
        let mut format = egui::TextFormat {
            font_id: font.clone(),
            color: colour(part, p),
            ..Default::default()
        };
        if err.as_ref().is_some_and(|r| r.contains(&a)) {
            format.underline = Stroke::new(1.5, p.danger);
        }
        if marks.contains(&a) {
            format.background = tint(p.accent, 70);
        }
        job.sections.push(egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: eframe::egui::text::ByteIndex(a)..eframe::egui::text::ByteIndex(b),
            format,
        });
    }
    ui.fonts_mut(|f| f.layout_job(job))
}
