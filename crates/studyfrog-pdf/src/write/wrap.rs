//! Word wrapping for styled Markdown text runs.

use studyfrog_md::blocks::Span;

use super::metrics::{char_width, measure_str};
use super::palette::{COLOR_LINK, COLOR_TEXT_BODY, hue_to_rgb};
use super::primitives::{StyledRun, TextLine};

/// Break a long word across multiple runs so that no part exceeds `max_width`.
fn split_long_word(word: &str, size: f32, bold: bool, mono: bool, max_w: f32) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_w = 0.0;
    for c in word.chars() {
        let cw = char_width(c, size, bold, mono);
        if current_w + cw > max_w && !current.is_empty() {
            chunks.push(current);
            current = String::new();
            current_w = 0.0;
        }
        current.push(c);
        current_w += cw;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Convert a Markdown [`Span`] into individual word tokens with style.
fn span_to_tokens(span: &Span, base_size: f32) -> Vec<(String, StyledRun)> {
    let is_code = span.style.code;
    let size = if is_code {
        (base_size * 0.85).max(7.0)
    } else {
        base_size
    };
    let color = if span.link.is_some() {
        COLOR_LINK
    } else if let Some(hue) = span.style.color {
        hue_to_rgb(hue)
    } else {
        COLOR_TEXT_BODY
    };
    let base_run = StyledRun {
        text: String::new(),
        bold: span.style.bold,
        italic: span.style.italic,
        underline: span.style.underline,
        strike: span.style.strike,
        code: is_code,
        color,
        font_size: size,
        width: 0.0,
        link: span.link.clone(),
    };

    let mut tokens = Vec::new();
    let words = span.text.split(' ');
    for (i, word) in words.enumerate() {
        if i > 0 {
            tokens.push((" ".to_string(), base_run.clone()));
        }
        if !word.is_empty() {
            tokens.push((word.to_string(), base_run.clone()));
        }
    }
    tokens
}

fn finish_line(runs: &mut Vec<StyledRun>, lines: &mut Vec<TextLine>, lh: f32) {
    if runs.is_empty() {
        return;
    }
    // Trim trailing whitespace from last run
    if let Some(last) = runs.last_mut() {
        let trimmed = last.text.trim_end();
        if trimmed.len() != last.text.len() {
            last.text = trimmed.to_string();
            last.width = measure_str(&last.text, last.font_size, last.bold, last.code);
        }
    }
    lines.push(TextLine {
        x: 0.0,
        y: 0.0,
        runs: std::mem::take(runs),
        height: lh,
        baseline: lh * 0.75,
    });
}

/// Wrap a sequence of styled spans into text lines not exceeding `max_width`.
pub fn wrap_spans(
    spans: &[Span],
    max_width: f32,
    font_size: f32,
    line_height: f32,
) -> Vec<TextLine> {
    let mut lines = Vec::new();
    let mut current_runs: Vec<StyledRun> = Vec::new();
    let mut current_w = 0.0;

    for span in spans {
        for (token_text, mut run) in span_to_tokens(span, font_size) {
            let is_space = token_text == " ";
            let token_w = measure_str(&token_text, run.font_size, run.bold, run.code);

            if is_space {
                if !current_runs.is_empty() && current_w + token_w <= max_width {
                    if let Some(last) = current_runs.last_mut().filter(|last| {
                        last.font_size == run.font_size
                            && last.bold == run.bold
                            && last.code == run.code
                    }) {
                        last.text.push(' ');
                        last.width += token_w;
                        current_w += token_w;
                        continue;
                    }
                    run.text = token_text;
                    run.width = token_w;
                    current_w += token_w;
                    current_runs.push(run);
                }
                continue;
            }

            if current_w + token_w <= max_width {
                run.text = token_text;
                run.width = token_w;
                current_w += token_w;
                current_runs.push(run);
            } else if current_w > 0.0 {
                finish_line(&mut current_runs, &mut lines, line_height);
                current_w = 0.0;
                if token_w <= max_width {
                    run.text = token_text;
                    run.width = token_w;
                    current_w = token_w;
                    current_runs.push(run);
                } else {
                    for chunk in
                        split_long_word(&token_text, run.font_size, run.bold, run.code, max_width)
                    {
                        let cw = measure_str(&chunk, run.font_size, run.bold, run.code);
                        let mut crun = run.clone();
                        crun.text = chunk;
                        crun.width = cw;
                        current_runs.push(crun);
                        finish_line(&mut current_runs, &mut lines, line_height);
                    }
                }
            } else {
                for chunk in
                    split_long_word(&token_text, run.font_size, run.bold, run.code, max_width)
                {
                    let cw = measure_str(&chunk, run.font_size, run.bold, run.code);
                    let mut crun = run.clone();
                    crun.text = chunk;
                    crun.width = cw;
                    current_runs.push(crun);
                    finish_line(&mut current_runs, &mut lines, line_height);
                }
            }
        }
    }

    finish_line(&mut current_runs, &mut lines, line_height);
    lines
}
