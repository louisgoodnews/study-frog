//! Split-screen PDF state management, page navigation, and ratio updates.

use studyfrog_pdf::crop::PdfCropAnchor;

use crate::effect::Effect;
use crate::pdf_types::{PdfEffect, PdfView, SplitPdfView};
use crate::state::State;

fn stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.rsplit_once('.').map_or(name, |(s, _)| s).to_owned()
}

/// Clamps target page index into valid `[0, total_pages - 1]`.
pub fn clamp_page_index(page: usize, total_pages: usize) -> usize {
    page.min(total_pages.saturating_sub(1))
}

/// Activate split-screen PDF pane and synchronize to target page coordinates.
pub fn open_crop(mut state: State, anchor: PdfCropAnchor) -> (State, Vec<Effect>) {
    let target_page = (anchor.page.saturating_sub(1)) as usize;
    state.split_pdf = SplitPdfView {
        active: true,
        anchor: Some(anchor.clone()),
        split_ratio: if state.split_pdf.split_ratio >= 0.2 {
            state.split_pdf.split_ratio
        } else {
            0.5
        },
        flash_start_now: state.now,
    };
    if let Some(ref mut view) = state.pdf
        && view.path == anchor.path
    {
        let total = view.info.as_ref().map_or(0, |i| i.page_count);
        let page = if total > 0 {
            clamp_page_index(target_page, total)
        } else {
            target_page
        };
        view.page = page;
        view.text = None;
        view.selection.clear();
        let ask = Effect::Pdf(PdfEffect::Text {
            path: view.path.clone(),
            page,
        });
        return (state, vec![ask]);
    }
    let path = anchor.path.clone();
    let view = PdfView {
        title: stem(&path),
        path: path.clone(),
        page: target_page,
        color: "yellow".to_owned(),
        ..PdfView::default()
    };
    state.pdf = Some(view);
    (state, vec![Effect::Pdf(PdfEffect::Open(path))])
}

/// Dismiss the split-screen PDF viewer pane.
pub fn close_split(mut state: State) -> (State, Vec<Effect>) {
    state.split_pdf.active = false;
    (state, Vec::new())
}

/// Adjust the split pane width ratio (clamped to 0.2..0.8).
pub fn set_split_ratio(mut state: State, ratio: f32) -> (State, Vec<Effect>) {
    state.split_pdf.split_ratio = ratio.clamp(0.2, 0.8);
    (state, Vec::new())
}
