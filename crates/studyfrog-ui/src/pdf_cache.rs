//! The PDF viewer's picture cache: a bounded LRU by last use, with pages and thumbnails kept apart
//! (STORY-0085). Pure.
//!
//! `view.pictures` is ordered from least to most recently used. A new picture goes to the end; a
//! picture drawn on screen is moved to the end by `PdfMsg::Used` when it has fallen into the older
//! half of its kind (`stale`), so a steady screen sends no messages at all. Trimming drops from the
//! front, so the pictures that go are those not used for the longest time, and a picture on screen
//! is never dropped (it is always in the newer half, which is larger than what one frame adds).

use std::sync::Arc;

use studyfrog_pdf::render::Pixels;

use crate::effect::Effect;
use crate::pdf_types::{MAX_PAGE_PICTURES, MAX_THUMB_PICTURES, PageKey, PdfView, is_thumb};
use crate::pdf_ui::with;
use crate::state::State;

/// The pictures of the cache, least recently used first.
pub type Pictures = Vec<(PageKey, Arc<Pixels>)>;

fn cap(thumb: bool) -> usize {
    if thumb {
        MAX_THUMB_PICTURES
    } else {
        MAX_PAGE_PICTURES
    }
}

/// Drop the least recently used pictures of one kind (thumbnails or full pages) beyond `cap`.
fn trim_kind(pictures: &mut Pictures, thumb: bool, cap: usize) {
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
pub fn trim(pictures: &mut Pictures) {
    trim_kind(pictures, false, cap(false));
    trim_kind(pictures, true, cap(true));
}

/// Whether the picture of `key` is held but has fallen into the older half of its kind, so that
/// drawing it should mark it as used again.
pub fn stale(view: &PdfView, key: PageKey) -> bool {
    let thumb = is_thumb(key);
    let newer = view
        .pictures
        .iter()
        .rev()
        .filter(|(k, _)| is_thumb(*k) == thumb)
        .position(|(k, _)| *k == key);
    newer.is_some_and(|n| n >= cap(thumb) / 2)
}

/// Move the picture of `key` to the most recently used end.
pub fn touch(pictures: &mut Pictures, key: PageKey) {
    if let Some(i) = pictures.iter().position(|(k, _)| *k == key) {
        let entry = pictures.remove(i);
        pictures.push(entry);
    }
}

/// `PdfMsg::Used`: the view drew these pictures.
pub(crate) fn used(state: State, keys: Vec<PageKey>) -> (State, Vec<Effect>) {
    let Some(mut view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    keys.into_iter().for_each(|k| touch(&mut view.pictures, k));
    (with(state, view), Vec::new())
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

    fn thumb(page: usize) -> PageKey {
        PageKey {
            page,
            scale_milli: THUMB_SCALE_MILLI,
        }
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
    fn the_least_recently_used_page_goes_first_not_the_oldest() {
        let mut pictures: Pictures = (0..MAX_PAGE_PICTURES).map(|p| pic(p, 1000)).collect();
        // Page 0 is the oldest, but it is read again, so page 1 is now the least recently used.
        touch(
            &mut pictures,
            PageKey {
                page: 0,
                scale_milli: 1000,
            },
        );
        pictures.push(pic(99, 1000));
        trim(&mut pictures);
        let pages: Vec<usize> = pictures.iter().map(|(k, _)| k.page).collect();
        assert!(pages.contains(&0), "the page used again stays: {pages:?}");
        assert!(
            !pages.contains(&1),
            "the least recently used goes: {pages:?}"
        );
        assert_eq!(pictures.len(), MAX_PAGE_PICTURES);
    }

    #[test]
    fn only_pictures_in_the_older_half_count_as_stale() {
        let view = PdfView {
            pictures: (0..MAX_THUMB_PICTURES)
                .map(|p| pic(p, THUMB_SCALE_MILLI))
                .collect(),
            ..PdfView::default()
        };
        assert!(stale(&view, thumb(0)));
        assert!(!stale(&view, thumb(MAX_THUMB_PICTURES - 1)));
        assert!(!stale(&view, thumb(MAX_THUMB_PICTURES / 2)));
        assert!(stale(&view, thumb(MAX_THUMB_PICTURES / 2 - 1)));
        assert!(!stale(&view, thumb(500)), "a picture not held is not stale");
    }

    #[test]
    fn a_thumbnail_kept_in_view_survives_any_amount_of_scrolling_elsewhere() {
        let mut view = PdfView::default();
        view.pictures.push(pic(7, THUMB_SCALE_MILLI));
        for page in 100..400 {
            view.pictures.push(pic(page, THUMB_SCALE_MILLI));
            trim(&mut view.pictures);
            if stale(&view, thumb(7)) {
                touch(&mut view.pictures, thumb(7));
            }
        }
        assert!(view.pictures.iter().any(|(k, _)| *k == thumb(7)));
        assert_eq!(view.pictures.len(), MAX_THUMB_PICTURES);
    }
}
