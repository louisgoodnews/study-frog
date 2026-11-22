//! The ScriptStudio text box (STORY-0150): FrogScript coloured as it is typed, the bracket matching the cursor marked,
//! the first error's line underlined and named, and the names that complete the word before the cursor offered as
//! buttons. The pieces are worked out by `studyfrog_script::editor`; this only draws them.

use std::sync::Arc;

use eframe::egui::{self, Color32, FontId, Galley, Stroke, TextStyle, text::LayoutJob};
use studyfrog_script::editor::{
    Part, completions, error_of, highlight, line_range, matching_bracket, with_completion,
};

use crate::console_ui::ConsoleMsg;
use crate::msg::Msg;
use crate::theme::{Palette, palette_of, tint};

/// The byte index of the cursor, as the editor had it at the last frame (kept so the layouter can mark a bracket).
#[derive(Clone, Copy)]
struct CursorAt(usize);

/// How many completions are offered at once.
const OFFERED: usize = 8;

/// The text box, its completions and its first error.
pub fn show(ui: &mut egui::Ui, code: &mut String, out: &mut Vec<Msg>) {
    let cursor = ui
        .data(|d| d.get_temp::<CursorAt>(egui::Id::NULL))
        .map(|c| c.0);
    let error = error_of(code);
    let error_line = error.as_ref().and_then(|e| e.line);
    let p = palette_of(ui);
    let font = TextStyle::Monospace.resolve(ui.style());
    let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap: f32| {
        layout(ui, text.as_str(), cursor, error_line, wrap, &p, &font)
    };
    let edit = egui::TextEdit::multiline(code)
        .id_salt("scriptstudio-multiline-text")
        .font(TextStyle::Monospace)
        .desired_rows(6)
        .desired_width(f32::INFINITY)
        .layouter(&mut layouter)
        .show(ui);
    let response = edit.response;
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Script text"));
    if let Some(cr) = edit.cursor_range {
        let chars = cr.primary.index.0;
        let at = code
            .char_indices()
            .nth(chars)
            .map_or(code.len(), |(b, _)| b);
        ui.data_mut(|d| d.insert_temp(egui::Id::NULL, CursorAt(at)));
    }
    if let Some(e) = &error {
        let line = e
            .line
            .map_or_else(|| "at the end".to_owned(), |l| format!("line {l}"));
        ui.label(egui::RichText::new(format!("{line}: {}", e.message)).color(p.danger));
    }
    if response.changed() {
        out.push(Msg::Console(ConsoleMsg::StudioEditCode(code.clone())));
    }
    completion_row(ui, code, cursor_now(ui), out);
}

/// The cursor after this frame's edit, falling back to the end of the text.
fn cursor_now(ui: &egui::Ui) -> usize {
    ui.data(|d| d.get_temp::<CursorAt>(egui::Id::NULL))
        .map_or(0, |c| c.0)
}

/// The names that can complete the word before the cursor, as buttons; a click puts the name in place of the word.
fn completion_row(ui: &mut egui::Ui, code: &str, cursor: usize, out: &mut Vec<Msg>) {
    let names = completions(code, cursor);
    if names.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.weak("Complete:");
        for name in names.iter().take(OFFERED) {
            if ui
                .small_button(name)
                .on_hover_text("Puts this name in place of the word before the cursor.")
                .clicked()
            {
                let (text, _) = with_completion(code, cursor, name);
                out.push(Msg::Console(ConsoleMsg::StudioEditCode(text)));
            }
        }
    });
}

fn colour(part: Part, p: &Palette) -> Color32 {
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
fn layout(
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
