//! The PDF viewer's loading side: pictures and words arriving, searching inside the file. Pure.

use std::sync::Arc;

use crate::effect::Effect;
use crate::pdf_types::{MAX_CACHED, PageKey, PdfEffect, PdfView, THUMB_SCALE_MILLI};
use crate::pdf_ui::{fail, go_to, with};
use crate::state::State;

pub(crate) fn need(state: State, key: PageKey) -> (State, Vec<Effect>) {
    let Some(mut view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let known = view.pictures.iter().any(|(k, _)| *k == key)
        || view.requested.contains(&key)
        || view.failed.contains(&key);
    if known {
        return (state, Vec::new());
    }
    view.requested.push(key);
    let ask = Effect::Pdf(PdfEffect::Render {
        path: view.path.clone(),
        key,
    });
    (with(state, view), vec![ask])
}

/// Drop pictures beyond the limit: thumbnails first, then the oldest.
fn trim(pictures: &mut Vec<(PageKey, Arc<studyfrog_pdf::render::Pixels>)>) {
    while pictures.len() > MAX_CACHED {
        let at = pictures
            .iter()
            .position(|(k, _)| k.scale_milli <= THUMB_SCALE_MILLI)
            .unwrap_or(0);
        pictures.remove(at);
    }
}

pub(crate) fn rendered(
    state: State,
    path: String,
    key: PageKey,
    result: Result<Arc<studyfrog_pdf::render::Pixels>, String>,
) -> (State, Vec<Effect>) {
    let Some(mut view) = state.pdf.clone().filter(|v| v.path == path) else {
        return (state, Vec::new());
    };
    view.requested.retain(|k| *k != key);
    match result {
        Ok(px) => {
            view.pictures.retain(|(k, _)| *k != key);
            view.pictures.push((key, px));
            trim(&mut view.pictures);
            (with(state, view), Vec::new())
        }
        Err(e) => {
            view.failed.push(key);
            view.error = (key.scale_milli > THUMB_SCALE_MILLI).then_some(e);
            (with(state, view), Vec::new())
        }
    }
}

pub(crate) fn text_loaded(
    state: State,
    path: String,
    page: usize,
    result: Result<Arc<studyfrog_pdf::text::PageText>, String>,
) -> (State, Vec<Effect>) {
    let Some(view) = state
        .pdf
        .clone()
        .filter(|v| v.path == path && v.page == page)
    else {
        return (state, Vec::new());
    };
    match result {
        Ok(text) => (
            with(
                state,
                PdfView {
                    text: Some(text),
                    ..view
                },
            ),
            Vec::new(),
        ),
        Err(e) => fail(
            state,
            format!("Could not read the text of page {}: {e}", page + 1),
        ),
    }
}

pub(crate) fn query(state: State, text: String) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let q = text.trim().to_owned();
    let ask = (!q.is_empty()).then(|| {
        Effect::Pdf(PdfEffect::Find {
            path: view.path.clone(),
            query: q.clone(),
        })
    });
    let found = if q.is_empty() {
        Vec::new()
    } else {
        view.found.clone()
    };
    (
        with(
            state,
            PdfView {
                query: text,
                found,
                ..view
            },
        ),
        ask.into_iter().collect(),
    )
}

pub(crate) fn found(
    state: State,
    path: String,
    q: String,
    result: Result<Vec<usize>, String>,
) -> (State, Vec<Effect>) {
    let Some(view) = state
        .pdf
        .clone()
        .filter(|v| v.path == path && v.query.trim() == q)
    else {
        return (state, Vec::new());
    };
    match result {
        Ok(pages) => {
            let jump = pages
                .iter()
                .copied()
                .find(|p| *p >= view.page)
                .or(pages.first().copied());
            let state = with(
                state,
                PdfView {
                    found: pages,
                    ..view
                },
            );
            jump.map_or((state.clone(), Vec::new()), |p| go_to(state, p))
        }
        Err(e) => fail(state, format!("Search failed: {e}")),
    }
}

pub(crate) fn hit(state: State, forward: bool) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let target = if forward {
        view.found
            .iter()
            .copied()
            .find(|p| *p > view.page)
            .or(view.found.first().copied())
    } else {
        view.found
            .iter()
            .rev()
            .copied()
            .find(|p| *p < view.page)
            .or(view.found.last().copied())
    };
    target.map_or((state.clone(), Vec::new()), |p| go_to(state, p))
}
