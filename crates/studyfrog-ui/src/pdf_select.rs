//! Picking words on a page with a dragged rectangle, and the text they make. Pure.

use studyfrog_pdf::render::PageRect;
use studyfrog_pdf::text::WordBox;

fn overlaps(w: &WordBox, r: &PageRect) -> bool {
    w.x < r.x + r.width && w.x + w.width > r.x && w.y < r.y + r.height && w.y + w.height > r.y
}

/// Indexes of the words the rectangle touches, in reading order of the page's word list.
pub fn words_in_rect(words: &[WordBox], rect: &PageRect) -> Vec<usize> {
    words
        .iter()
        .enumerate()
        .filter(|(_, w)| overlaps(w, rect))
        .map(|(i, _)| i)
        .collect()
}

/// The selected words as text: spaces between words, a line break where the next word is on a new line.
pub fn selected_text(words: &[WordBox], selection: &[usize]) -> String {
    let mut out = String::new();
    let mut last: Option<&WordBox> = None;
    for w in selection.iter().filter_map(|i| words.get(*i)) {
        match last {
            Some(p) if w.y > p.y + p.height * 0.6 => out.push('\n'),
            Some(_) => out.push(' '),
            None => {}
        }
        out.push_str(&w.text);
        last = Some(w);
    }
    out
}

/// Indexes of the words that contain any word of `query` (case-insensitive).
pub fn matching_words(words: &[WordBox], query: &str) -> Vec<usize> {
    let tokens: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if tokens.is_empty() {
        return Vec::new();
    }
    words
        .iter()
        .enumerate()
        .filter(|(_, w)| {
            let t = w.text.to_lowercase();
            tokens.iter().any(|q| t.contains(q.as_str()))
        })
        .map(|(i, _)| i)
        .collect()
}

/// One rectangle per line of the selected words, each as wide as its words.
pub fn line_rects(
    words: &[WordBox],
    selection: &[usize],
) -> Vec<studyfrog_config::pdf_marks::MarkRect> {
    use studyfrog_config::pdf_marks::MarkRect;
    let mut rects: Vec<MarkRect> = Vec::new();
    for w in selection.iter().filter_map(|i| words.get(*i)) {
        match rects.last_mut() {
            Some(r) if (w.y - r.y).abs() < r.height * 0.6 => {
                let right = (r.x + r.width).max(w.x + w.width);
                let bottom = (r.y + r.height).max(w.y + w.height);
                r.x = r.x.min(w.x);
                r.y = r.y.min(w.y);
                r.width = right - r.x;
                r.height = bottom - r.y;
            }
            _ => rects.push(MarkRect {
                x: w.x,
                y: w.y,
                width: w.width,
                height: w.height,
            }),
        }
    }
    rects
}
