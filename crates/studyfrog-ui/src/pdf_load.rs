//! The PDF viewer's loading side: pictures and words arriving, searching inside the file. Pure.

use std::sync::Arc;

use crate::effect::Effect;
use crate::pdf_types::{
    MAX_PAGE_PICTURES, MAX_THUMB_PICTURES, PageKey, PdfEffect, PdfView, is_thumb,
};
use crate::pdf_ui::{fail, go_to, with};
use crate::state::State;
use studyfrog_pdf::render::Pixels;

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

/// Drop the oldest pictures of one kind (thumbnails or full pages) beyond `cap`.
fn trim_kind(pictures: &mut Vec<(PageKey, Arc<Pixels>)>, thumb: bool, cap: usize) {
    let mut extra = pictures
        .iter()
        .filter(|(k, _)| is_thumb(*k) == thumb)
        .count()
        .saturating_sub(cap);
    pictures.retain(|(k, _)| {
        if is_thumb(*k) != thumb || extra == 0 {
            return true;
        }
        extra -= 1;
        false
    });
}

/// Keep pages and thumbnails within their own limits, so that one kind cannot push out the other.
fn trim(pictures: &mut Vec<(PageKey, Arc<Pixels>)>) {
    trim_kind(pictures, false, MAX_PAGE_PICTURES);
    trim_kind(pictures, true, MAX_THUMB_PICTURES);
}

pub(crate) fn rendered(
    state: State,
    path: String,
    key: PageKey,
    result: Result<Arc<Pixels>, String>,
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
            view.error = (!is_thumb(key)).then_some(e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf_types::THUMB_SCALE_MILLI;

    fn pic(page: usize, scale_milli: u32) -> (PageKey, Arc<Pixels>) {
        let px = Pixels {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        };
        (PageKey { page, scale_milli }, Arc::new(px))
    }

    #[test]
    fn scrolling_thumbnails_never_pushes_out_the_pages_being_read() {
        let mut pictures = vec![pic(0, 1250), pic(1, 1250)];
        for page in 0..200 {
            pictures.push(pic(page, THUMB_SCALE_MILLI));
            trim(&mut pictures);
        }
        let pages = pictures.iter().filter(|(k, _)| !is_thumb(*k)).count();
        assert_eq!(pages, 2);
        assert_eq!(pictures.len() - pages, MAX_THUMB_PICTURES);
    }

    #[test]
    fn the_oldest_page_pictures_go_first_and_the_newest_stay() {
        let mut pictures: Vec<_> = (0..MAX_PAGE_PICTURES + 3).map(|p| pic(p, 1000)).collect();
        trim(&mut pictures);
        assert_eq!(pictures.len(), MAX_PAGE_PICTURES);
        assert_eq!(pictures[0].0.page, 3);
        assert_eq!(
            pictures.last().map(|(k, _)| k.page),
            Some(MAX_PAGE_PICTURES + 2)
        );
    }
}
